use std::net::TcpStream;
use crate::response::{Response, ResponseCode, ResponseBody};
use crate::request::{Request, RE_SHORT_URL_VALIDATE, RE_LONG_URL_VALIDATE, RequestBody};
use crate::database::Database;
use rand::Rng;

pub const RESERVED_URLS: [&str; 5] = [
    "create",
    "free",
    "delete",
    "status",
    "config"
];


pub fn create_page(mut s: TcpStream, req: Request, db: &dyn Database) -> std::io::Result<()> {
    match req.body {
        Some(RequestBody::FormUrlEncoded(map)) => {
            if let (Some(code), Some(long)) = (map.get("password"), map.get("long-url")) {
                if db.is_password(code) && validate_long_url(long).is_ok() {
                    // pw correct and long url valid
                    let short = map.get("short-url").map_or_else(|| gen_free_random_url(db),|s| s.to_owned());
                }
            }
        },
        None => {}
    }
    send_gen_error_page(s, "No Request transmitted")
}

fn gen_free_random_url(db: &dyn Database) -> String {
    let mut rg = rand::thread_rng();
    let allowed_chars = ('a'..'z')
        .chain('A'..'Z')
        .chain('0'..'9').cylce();

    const ALLOWED_CHARS_LEN: usize = 26 * 2 + 10;

    for _ in 0..100 {
        let res: String = (0..5).next().map(|_|
            allowed_chars.nth(rg.gen_range(0, ALLOWED_CHARS_LEN)).unwrap()
        ).collect();
        if validate_short_url(&res) {
            return res;
        }
    }
    panic!("Not enough free short urls!")
}

pub fn home_page(mut s: TcpStream) -> std::io::Result<()> {
    let page = std::fs::read_to_string("./page/dist/index.html").unwrap();
    let r = Response{
        code: ResponseCode::Ok,
        body: ResponseBody::Html(page)
    };
    r.write_html11(&mut s)
}

pub fn send_gen_error_page(mut s: TcpStream, error: &str) -> std::io::Result<()> {
    let mut page = std::fs::read_to_string("./page/dist/400.html").unwrap();

    page = page.replace("{{error}}", error);

    let r = Response{
        code: ResponseCode::BadRequest,
        body: ResponseBody::Html(page)
    };
    r.write_html11(&mut s)
}

pub fn send_404_page(mut s: TcpStream, req: Request) -> std::io::Result<()> {
    let mut page = std::fs::read_to_string("./page/dist/404.html").unwrap();

    page = page.replace("{{url}}", &req.url.join("/"));

    let r = Response{
        code: ResponseCode::NotFound,
        body: ResponseBody::Html(page)
    };
    r.write_html11(&mut s)
}

pub fn validate_short_url(short: &str) -> bool {
    if short.len() < 3 { return false; }
    if !RE_SHORT_URL_VALIDATE.as_ref().unwrap().is_match(&short) {
        return false;
    }

    if RESERVED_URLS.iter().any(|ru| ru.eq_ignore_ascii_case(&short)){
        return false;
    }
    // TODO saved urls
    true
}

enum ValidationResult {
    Ok,
    NoUrl,
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

fn validate_long_url(long: &str) -> ValidationResult {
    if long.len() < 3 || !RE_LONG_URL_VALIDATE.as_ref().unwrap().is_match(&long) {
        return ValidationResult::NoUrl;
    }
    ValidationResult::Ok
}

pub fn free_check(mut s: TcpStream, req: Request) -> std::io::Result<()> {
    println!("{:?}", req.params);
    // TODO check long urls
    let mut rcode = ResponseCode::NotAcceptable;
    let mut rbody = ResponseBody::Empty;
    if let Some(short) = req.params.get("short") {
        if validate_short_url(short) {
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
        body: rbody
    };
    res.write_html11(&mut s)
}

pub fn static_content(mut s: TcpStream, req: Request) -> std::io::Result<()> {

    let mut path: String = req.url[1..].join("/");
    if path.contains("..") {
        return send_404_page(s, req);
    }
    path.insert_str(0, "./page/dist/");


    let r = Response{
        code: ResponseCode::Ok,
        body: ResponseBody::load_from_file(&path)?
    };
    r.write_html11(&mut s)
}
