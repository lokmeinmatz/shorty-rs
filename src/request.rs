use std::collections::HashMap;
use std::convert::TryFrom;
use regex::Regex;
use crate::{log, DEBUG_VERBOSE};
use std::io::{BufReader, BufRead, Read};
use std::net::{TcpStream, IpAddr};
use std::sync::atomic::Ordering;

pub static RE_GET_HEADER: Option<Regex> = None;
pub static RE_SHORT_URL_VALIDATE: Option<Regex> = None;
pub static RE_LONG_URL_VALIDATE: Option<Regex> = None;


// TODO change to safe variant
pub(crate) fn init_regex() {

    log("Init regex");

    unsafe {
        *(&RE_GET_HEADER as *const _ as *mut _) = Some(Regex::new(r"(GET|POST) /([^?\s]*/?)*(?:\?(\S+=\S+)+)? HTTP/1\.1").unwrap());
        *(&RE_SHORT_URL_VALIDATE as *const _ as *mut _) = Some(Regex::new(r"^[\w\d|\-|_]{3,}$").unwrap());
        // TODO valid url regex
        *(&RE_LONG_URL_VALIDATE as *const _ as *mut _) = Some(Regex::new(r"[^<>]{3,}").unwrap());
    }
}

/// checks if c is a "hex char", eg. 0-9, a-f, A-F
#[inline]
fn is_hex_char(c: char) -> bool {

    ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}


/// Decodes an url encoded ASCII string
pub fn decode_url_str(mut encoded: &str) -> Option<String> {
    let mut result = String::with_capacity(encoded.len());
    while let Some(next_perc_idx) = encoded.find("%") {
        // copy until %
        result.push_str(&encoded[..next_perc_idx]);
        // TODO utf8
        //println!("{} next % at {} {}", encoded, next_perc_idx, &encoded[next_perc_idx+1..next_perc_idx + 3]);
        if !is_hex_char(encoded.as_bytes()[next_perc_idx + 1] as char) ||
            !is_hex_char(encoded.as_bytes()[next_perc_idx + 2] as char) {
            return None;
        }
        let ascii_num = u8::from_str_radix(&encoded[next_perc_idx+1..next_perc_idx+3], 16).ok()?;
        result.push(ascii_num as char);
        encoded = &encoded[next_perc_idx+3..];
    }

    result.push_str(encoded);
    Some(result)
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post
}

impl TryFrom<&str> for Method {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.eq_ignore_ascii_case("get") {
            Ok(Method::Get)
        } else if value.eq_ignore_ascii_case("post") {
            Ok(Method::Post)
        } else { Err(()) }
    }
}

/// The content if the client provided an request body
/// Currently only accepting x-form-url-encoded
#[derive(Debug)]
pub enum RequestBody {
    FormUrlEncoded(HashMap<String, String>)
}

/// The request send from the client
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: Box<[String]>,
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>,
    pub ip: IpAddr
}

impl Request {

    /// Returns an info- `String` about the request, containing method and url-path
    #[allow(dead_code)]
    pub fn basic_info(&self) -> String {
        format!("Request {{ {:?} {} }}", self.method, self.url.join("/"))
    }
}


impl TryFrom<&mut BufReader<TcpStream>> for Request {
    type Error = &'static str;

    fn try_from(s: &mut BufReader<TcpStream>) -> Result<Self, Self::Error> {
        let mut buffer = String::with_capacity(1024);

        let debug = DEBUG_VERBOSE.load(Ordering::Relaxed);

        let linescan = s.read_line(&mut buffer);

        if linescan.is_err() || linescan.unwrap() == 0 {
            return Err("No http header");
        }

        // parse first line
        let (method, url, query): (Method, Vec<String>, HashMap<String, String>) = {
            //log(l);
            let l = buffer.as_str();
            if debug { print!("{}", l); }
            let matches = RE_GET_HEADER.as_ref().unwrap().captures(l).ok_or("no http header")?;
            let method_match = matches.get(1).ok_or("no method match")?;
            let url_match: Vec<String> = matches.get(2)
                .map_or(vec![], |m|
                    m.as_str().split("/").map(|p| p.into()).collect()
                );
            let method = Method::try_from(method_match.as_str()).map_err(|_| "unknown method")?;
            let mut q = HashMap::new();

            if let Some(qm) = matches.get(3) {
                for q_str in qm.as_str().split("&") {
                    let mut sq_iter = q_str.split("=");
                    let q_key = sq_iter.next().unwrap();
                    let q_value = sq_iter.next().unwrap();
                    q.insert(q_key.into(), q_value.into());
                }
            }

            (method, url_match, q)
        };


        // parse headers
        let mut headers = HashMap::new();
        buffer.clear();
        while let Ok(l) = s.read_line(&mut buffer) {
            if l == 0 || buffer.trim().is_empty() { break }
            if debug { println!("{}", buffer); }
            if let Some(split_idx) = buffer.find(":") {
                let name = (&buffer[0..split_idx]).trim();
                let value = (&buffer[split_idx + 1 ..]).trim();
                headers.insert(name.into(), value.into());
            }
            buffer.clear();
        }
        let mut body = None;

        // parse request body (form)
        if method == Method::Post && headers.get("Content-Type").map_or(false,
                    |ct: &String| ct.eq_ignore_ascii_case("application/x-www-form-urlencoded")) {
            if let Ok(l) = headers.get("Content-Length").map_or(Ok(0usize), |s| s.parse::<usize>()) {

                // read form str
                let mut buffer = vec![0u8; l];
                if s.read_exact(&mut buffer).is_ok() {
                    if let Ok(body_str) = std::str::from_utf8(&buffer) {

                        let mut payload = HashMap::new();
                        for (key, val) in body_str.split("&").map(|u| {
                            let sidx = u.find("=").unwrap_or(0);
                            let k = decode_url_str(&u[0..sidx]).unwrap();
                            let v = decode_url_str(&u[sidx + 1..]).unwrap();

                            (k, v)
                        }) {
                            payload.insert(key, val);
                        }
                        body = Some(RequestBody::FormUrlEncoded(payload));


                    }
                }

            }

        }

        //println!("{:?} {:?} {:?}", method, url, query);

        Ok(Self {
            method,
            url: url.into_boxed_slice(),
            params: query,
            headers,
            body,
            ip: s.get_ref().peer_addr().unwrap().ip()
        })
    }
}