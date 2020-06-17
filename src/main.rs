#![feature(osstring_ascii)]
#![feature(result_flattening)]

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

//const PORT: usize = 80; // production port


fn main() -> Result<(), ()> {
    log("Started Shorty-rs");

    let base_url = std::env::var("SHORTY_BASE_URL").expect("Set SHORTY_BASE_URL");
    let database_path = std::env::var("SHORTY_DB_PATH").expect("Set SHORTY_DB_PATH");
    let port: u16 = std::env::var("SHORTY_PORT").ok().map(|s| s.parse().ok()).flatten().unwrap_or(80);

    log(format!("Base address: {}", base_url));

    request::init_regex();

    log("Starting listener");
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).unwrap();

    log("connecting to database");

    let mut db = database::SQLiteDB::init_database(&database_path)
        .expect("Database init failed");


    log(format!("Listening on port {:?}", listener.local_addr().unwrap().port()));

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            handle_request(BufReader::new(s), db.as_mut()).or_else(|e| {
                log(format!("handling failed: {:?}", e));
                Ok(())
            })?;
        }
    }

    Ok(())
}


fn handle_request(mut s: BufReader<TcpStream>, db: &mut dyn Database) -> std::io::Result<()> {

    let req = match Request::try_from(&mut s) {
        Ok(r) => r,
        Err(e) => { log(e); return Err(ErrorKind::InvalidData.into()); }
    };

    // routing
    for (_route_name, test, handle_fn) in &handler::HANDLERS {
        if test(&req) {
            //log(format!("Handling {} with {}", req.basic_info(), route_name));
            // handle_fn can either return the valid response, or diffrent error codes
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

                // force browser to use no-cache to allow counting of redirects
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
