use std::net::TcpStream;
use crate::response::{Response, ResponseCode, ResponseBody};
use crate::request::{Request, RE_SHORT_URL_VALIDATE};
use regex::Regex;

pub const RESERVED_URLS: [&str; 5] = [
    "create",
    "free",
    "delete",
    "status",
    "config"
];


pub fn send_create_page(mut s: TcpStream) -> std::io::Result<()> {
    let page = std::fs::read_to_string("./page/dist/index.html").unwrap();
    let r = Response{
        code: ResponseCode::Ok,
        body: ResponseBody::Html(page)
    };
    r.write_html11(&mut s)
}

pub fn send_error_page(mut s: TcpStream, req: Request) -> std::io::Result<()> {
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

pub fn free_check(mut s: TcpStream, req: Request) -> std::io::Result<()> {
    println!("{:?}", req.params);
    // TODO check long urls
    match req.params.get("short") {
        Some(short) => {
            if validate_short_url(short) {
                let res = Response {
                    code: ResponseCode::Ok,
                    body: ResponseBody::Empty
                };
                return res.write_html11(&mut s);
            }
        },
        None => {}
    }
    let res = Response {
        code: ResponseCode::NotAcceptable,
        body: ResponseBody::Empty
    };
    res.write_html11(&mut s)
}

pub fn static_content(mut s: TcpStream, req: Request) -> std::io::Result<()> {

    let mut path: String = req.url[1..].join("/");
    if path.contains("..") {
        return send_error_page(s, req);
    }
    path.insert_str(0, "./page/dist/");


    let r = Response{
        code: ResponseCode::Ok,
        body: ResponseBody::load_from_file(&path)?
    };
    r.write_html11(&mut s)
}
