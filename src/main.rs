#![feature(osstring_ascii)]

use std::net::{TcpListener, SocketAddr, TcpStream};
use chrono::prelude::*;
use std::io::{Read, ErrorKind, Error};
use std::convert::TryFrom;

mod request;
use request::*;
use crate::database::Database;

mod response;

mod handler;

mod database;

pub(crate) fn log<T: AsRef<str>>(msg: T) {
    println!("[{:?}] {}", Local::now(), msg.as_ref());
}

const PORT: u16 = 7070;
//const PORT: usize = 80; // production port


fn main() {
    log("Started Shorty-rs");

    request::init_regex();

    log("Starting listener");
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT))).unwrap();

    log("connectiong to database");

    let db = database::SQLiteDB::init_database("./database.sqlite")
        .expect("Database init failed");


    log(format!("Listening on addr {:?}", listener.local_addr()));

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            handle_request(s, db.as_ref()).or_else(|e| {
                log(format!("handling failed: {:?}", e));
                Err(())
            });
        }
    }
}


fn handle_request(mut s: TcpStream, db: &dyn Database) -> std::io::Result<()> {
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
        return handler::home_page(s);
    }
    else if req.url.len() > 0 && req.url[0].eq_ignore_ascii_case("static") {
        return handler::static_content(s, req);
    }
    else if req.url.len() > 0 && req.url[0].eq_ignore_ascii_case("free") {
        return handler::free_check(s, req);
    }
    else if req.url.len() > 0 && req.url[0].eq_ignore_ascii_case("create") {
        return handler::create_page(s, req, db);
    }

    println!("unknown req: {:?}", req.url);
    handler::send_404_page(s, req)
}
