#![feature(osstring_ascii)]

use std::net::{TcpListener, SocketAddr, TcpStream};
use chrono::prelude::*;
use std::io::{ErrorKind, BufReader};
use std::convert::TryFrom;

mod request;
use request::*;
use crate::database::Database;
use crate::handler::HandlerError;
use crate::response::{Response, ResponseCode, ResponseBody};
use std::collections::HashMap;

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

    let base_url = std::env::var("SHORTY_BASE_URL").expect("Set SHORTY_BASE_URL");

    log(format!("Base address: {}", base_url));

    request::init_regex();

    log("Starting listener");
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT))).unwrap();

    log("connecting to database");

    let mut db = database::SQLiteDB::init_database("./database.sqlite")
        .expect("Database init failed");


    log(format!("Listening on addr {:?}", listener.local_addr()));

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            handle_request(BufReader::new(s), db.as_mut()).or_else(|e| {
                log(format!("handling failed: {:?}", e));
                Err(())
            });
        }
    }
}


fn handle_request(mut s: BufReader<TcpStream>, db: &mut dyn Database) -> std::io::Result<()> {

    // TODO read until linefeed
    let req = match Request::try_from(&mut s) {
        Ok(r) => r,
        Err(e) => { log(e); return Err(ErrorKind::InvalidData.into()); }
    };

    //log(format!("Got request {}", req.basic_info()));

    // routing
    for (route_name, test, handle_fn) in &handler::HANDLERS {
        if test(&req) {
            //log(format!("Handling {} with {}", req.basic_info(), route_name));
            return match handle_fn(&req, db) {
                Ok(res) => {
                    res.write_html11(s.get_mut())
                }
                Err(HandlerError::E400(emsg)) => {
                    log(&emsg);
                    handler::send_gen_error_page(s.get_mut(), &emsg)
                },
                Err(HandlerError::E404) => {
                    handler::send_404_page(s.get_mut(), &req)
                },
                Err(HandlerError::Custom(r)) => {
                    r.write_html11(s.get_mut())
                }
            }
        }
    }
    if req.url.len() == 1 {
        // short url routing
        match db.forward(&req.url[0]) {
            Ok(long_url) => {
                // forward
                let mut h = HashMap::new();
                h.insert("Location".into(), long_url);
                h.insert("Cache-Control".into(), "no-cache".into());
                return Response {
                    code: ResponseCode::MovedPermanently,
                    custom_headers: Some(h),
                    body: ResponseBody::Empty
                }.write_html11(s.get_mut())
            },
            Err(()) => {}
        }

    }


    println!("unknown req: {:?}", req.url);
    handler::send_404_page(s.get_mut(), &req)
}
