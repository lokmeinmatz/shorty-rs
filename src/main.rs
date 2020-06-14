#![feature(osstring_ascii)]

use std::net::{TcpListener, SocketAddr, TcpStream};
use chrono::prelude::*;
use std::io::{Read, ErrorKind, Error};
use std::convert::TryFrom;

mod request;
use request::*;

mod response;
use response::*;

mod handler;

pub(crate) fn log<T: AsRef<str>>(msg: T) {
    println!("[{:?}] {}", Local::now(), msg.as_ref());
}

const PORT: u16 = 7070;
//const PORT: usize = 80; // production port


fn main() {
    log("Started Shorty-rs");

    request::init_regex();

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT))).unwrap();




    log(format!("Listening on addr {:?}", listener.local_addr()));

    for stream in listener.incoming() {
        let res: Result<(), ()> = stream.and_then(handle_request)
            .or_else(|e| Ok(log(format!("handling failed: {:?}", e))));
    }
}


fn handle_request(mut s: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 512];

    s.read(&mut buffer)?;
    let raw_req = std::str::from_utf8(&buffer).or_else(|e| Err(Error::from(ErrorKind::InvalidData)))?;
    let req = match Request::try_from(raw_req) {
        Ok(r) => r,
        Err(e) => { log(e); return Err(ErrorKind::InvalidData.into()); }
    };

    log(format!("Got request {}", req.basic_info()));

    // routing
    if req.url.len() == 0 && req.method == Method::Get {
        return handler::send_create_page(s);
    }
    else if req.url.len() > 0 && req.url[0].eq_ignore_ascii_case("static") {
        return handler::static_content(s, req);
    }
    else if req.url.len() > 0 && req.url[0].eq_ignore_ascii_case("free") {
        return handler::free_check(s, req);
    }
    println!("unknown req: {:?}", req.url);
    return handler::send_error_page(s, req);
    //unimplemented!();
    Ok(())
}
