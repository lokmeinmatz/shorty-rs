use std::net::TcpStream;
use crate::response::{Response, ResponseCode, ResponseBody};
use crate::request::{Request, RE_SHORT_URL_VALIDATE, RE_LONG_URL_VALIDATE, RequestBody, Method};
use crate::database::Database;
use rand::Rng;
use crate::log;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// the urls that are forbidden to use
pub const RESERVED_URLS: [&str; 5] = [
    "create",
    "free",
    "delete",
    "status",
    "config"
];


/// Contains all "static" handlers with fixed route (could also contain shortys)
/// They are tested top-to-bottom
/// if the test method is true, the handler is executed
pub const HANDLERS: [(&'static str, RoutingFn, HandlerFn); 4] = [
    // home page
    ("home_page", |req| req.url.len() == 0 && req.method == Method::Get
     , home_page),
    // create page
    ("create_page", |req| req.url.len() == 1 && req.url[0].eq_ignore_ascii_case("create"),
    create_page),
    //
    ("free_check", |req| req.url.len() == 1 && req.url[0].eq_ignore_ascii_case("free"),
     free_check),
    // static
    ("static", |req| req.url.len() > 1 && req.url[0].eq_ignore_ascii_case("static"), static_content)
];

type RoutingFn = fn(req: &Request) -> bool;

pub enum HandlerError {
    E404,
    E400(String),
    #[allow(dead_code)]
    Custom(Response)
}

type HandlerFn = fn(req: &Request, db: &dyn Database) -> Result<Response, HandlerError>;


/// the post endpoint for a page-create-request
fn create_page(req: &Request, db: &dyn Database) -> Result<Response, HandlerError> {
    log(format!("create {:?}", req.body));
    match &req.body {
        Some(RequestBody::FormUrlEncoded(map)) => {
            if let (Some(code), Some(long)) = (map.get("password"), map.get("long-url")) {

                let mut long = long.clone();
                if !long.starts_with("https://") && !long.starts_with("http://") {
                    long.insert_str(0, "https://");
                }

                // TODO do we need transactions?
                if db.is_password(code) && validate_long_url(&long).is_ok() {
                    // pw correct and long url valid
                    let short = map.get("short-url").map_or_else(|| gen_free_random_url(db),|s| s.to_owned());
                    if db.peek_long_url(&short).is_ok() {
                        return Err(HandlerError::E400(format!("Short URL {} allready exists.", &short)));
                    }
                    let ip_hash = {
                        let mut h = DefaultHasher::new();
                        req.ip.hash(&mut h);
                        h.finish() as u32
                    };

                    // check if user contingent is maxed out
                    let urls_created = db.urls_stored_last_7_days(ip_hash);
                    log(format!("Urls created by ith ip: {}", urls_created));

                    if urls_created > 100 {
                        return Err(HandlerError::E400("You allready created 100 URLs in the last 7 days... Thats a lot!".into()));
                    }

                    log(format!("Storing {} -> {}", &short, long));
                    db.store_shortened(&long, &short, ip_hash).map_err(|e| HandlerError::E400(e))?;

                    let mut file = std::fs::read_to_string("./page/dist/created.html").unwrap();
                    let host_url = std::env::var("SHORTY_BASE_URL").unwrap();
                    file = file.replace("{{short-url}}", &format!("{}/{}", host_url, short));
                    file = file.replace("{{long-url}}", &long);
                    return Ok(Response{
                        code: ResponseCode::Ok,
                        custom_headers: None,
                        body: ResponseBody::Html(file)
                    })
                }
            }
        },
        None => {}
    }
    Err(HandlerError::E400("No body transmitted".into()))
}

/// generates a free (valid) short url
fn gen_free_random_url(db: &dyn Database) -> String {
    let mut rg = rand::thread_rng();
    let mut allowed_chars = (b'a'..b'z')
        .chain(b'A'..b'Z')
        .chain(b'0'..b'9').cycle();

    const ALLOWED_CHARS_LEN: usize = 26 * 2 + 10;

    for _ in 0..100 {
        let res: String = (0..5).map(|_|
            allowed_chars.nth(rg.gen_range(0, ALLOWED_CHARS_LEN)).unwrap() as char
        ).collect();
        if validate_short_url(&res, db) {
            return res;
        }
    }
    panic!("Not enough free short urls!")
}


/// home page get (static)
pub fn home_page(_req: &Request, _db: &dyn Database) -> Result<Response, HandlerError> {
    let page = std::fs::read_to_string("./page/dist/index.html").unwrap();
    let r = Response{
        code: ResponseCode::Ok,
        custom_headers: None,
        body: ResponseBody::Html(page)
    };
    Ok(r)
}


/// generic error page (dynamic)
pub fn send_gen_error_page(s: &mut TcpStream, error: &str) -> std::io::Result<()> {
    let mut page = std::fs::read_to_string("./page/dist/400.html").unwrap();

    page = page.replace("{{error}}", error);

    let r = Response{
        code: ResponseCode::BadRequest,
        custom_headers: None,
        body: ResponseBody::Html(page)
    };
    r.write_html11(s)
}

/// 404 page (dynamic)
pub fn send_404_page(s: &mut TcpStream, req: &Request) -> std::io::Result<()> {
    if let Some(accpt) = req.headers.get("Accept") {
        if accpt.contains("text/html") {
            let mut page = std::fs::read_to_string("./page/dist/404.html").unwrap();

            page = page.replace("{{url}}", &req.url.join("/"));

            let r = Response{
                code: ResponseCode::NotFound,
                custom_headers: None,
                body: ResponseBody::Html(page)
            };
            return r.write_html11(s);
        }
    }

    let r = Response{
        code: ResponseCode::NotFound,
        custom_headers: None,
        body: ResponseBody::Empty
    };
    r.write_html11(s)
}

/// check if short url is free
pub fn validate_short_url(short: &str, db: &dyn Database) -> bool {
    if short.len() < 3 { return false; }
    if !RE_SHORT_URL_VALIDATE.as_ref().unwrap().is_match(&short) {
        return false;
    }

    if RESERVED_URLS.iter().any(|ru| ru.eq_ignore_ascii_case(&short)){
        return false;
    }
    db.peek_long_url(short).is_err()
}

enum ValidationResult {
    Ok,
    NoUrl,
    #[allow(dead_code)]
    InUse
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        match self {
            ValidationResult::Ok => true,
            _ => false
        }
    }
}

/// validate long url, fail if html injection etc.
// TODO better validation
fn validate_long_url(long: &str) -> ValidationResult {
    if long.len() < 3 || !RE_LONG_URL_VALIDATE.as_ref().unwrap().is_match(&long) {
        return ValidationResult::NoUrl;
    }
    ValidationResult::Ok
}

/// Endpoint for "while typing" to give a preview if ths is available
/// format: /free?short=...
pub fn free_check(req: &Request, db: &dyn Database) -> Result<Response, HandlerError> {
    //println!("{:?}", req.params);
    // TODO check long urls
    let mut rcode = ResponseCode::NotAcceptable;
    let mut rbody = ResponseBody::Empty;
    if let Some(short) = req.params.get("short") {
        if validate_short_url(short, db) {
            rcode = ResponseCode::Ok;
            rbody = ResponseBody::Empty;
        }
    }
    if let Some(long) = req.params.get("long") {
        match validate_long_url(long) {
            ValidationResult::Ok => {
                rcode = ResponseCode::Ok;
                rbody = ResponseBody::Empty;
            },
            ValidationResult::NoUrl => {
                rcode = ResponseCode::NotAcceptable;
                rbody = ResponseBody::Custom {
                    content_type: "text/plain".into(),
                    data: "This is no valid url to store.".bytes().collect()
                };
            },
            ValidationResult::InUse => {
                rcode = ResponseCode::NotAcceptable;
                rbody = ResponseBody::Custom {
                    content_type: "text/plain".into(),
                    data: "This URL is allready in use!".bytes().collect()
                }
            }
        }

    }
    let res = Response {
        code: rcode,
        custom_headers: None,
        body: rbody
    };
    Ok(res)
}

pub fn static_content(req: &Request, _db: &dyn Database) -> Result<Response, HandlerError> {

    let mut path: String = req.url[1..].join("/");
    if path.contains("..") {
        return Err(HandlerError::E404);
    }
    path.insert_str(0, "./page/dist/");


    let r = Response{
        code: ResponseCode::Ok,
        custom_headers: None,
        body: ResponseBody::load_from_file(&path).map_err(|_| HandlerError::E404)?
    };
    Ok(r)
}
