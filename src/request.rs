use std::collections::HashMap;
use std::convert::TryFrom;
use regex::Regex;
use crate::log;

pub static RE_GET_HEADER: Option<Regex> = None;
pub static RE_SHORT_URL_VALIDATE: Option<Regex> = None;
pub static RE_LONG_URL_VALIDATE: Option<Regex> = None;

pub(crate) fn init_regex() {

    log("Init regex");

    unsafe {
        *(&RE_GET_HEADER as *const _ as *mut _) = Some(Regex::new(r"(GET|POST) /([^?\s]*/?)*(?:\?(\S+=\S+)+)? HTTP/1\.1").unwrap());
        *(&RE_SHORT_URL_VALIDATE as *const _ as *mut _) = Some(Regex::new(r"^[\w\d|\-|_]{3,}$").unwrap());
        // TODO valid url regex
        *(&RE_LONG_URL_VALIDATE as *const _ as *mut _) = Some(Regex::new(r"^(?:http[s]?://)?").unwrap());
    }
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


#[derive(Debug)]
pub enum RequestBody {
    FormUrlEncoded(HashMap<String, String>)
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: Box<[String]>,
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<RequestBody>
}

impl Request {
    pub fn basic_info(&self) -> String {
        format!("Request {{ {:?} {} }}", self.method, self.url.join("/"))
    }
}


impl TryFrom<&str> for Request {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        println!("{}", s);
        let mut lines = s.lines();



        let (method, url, query): (Method, Vec<String>, HashMap<String, String>) = if let Some(l) = lines.next() {
            //log(l);
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
        } else { return Err("no content"); };

        let mut headers = HashMap::new();

        while let Some(l) = lines.next() {
            if l.is_empty() { break }

            if let Some(split_idx) = l.find(":") {
                let name = (&l[0..split_idx]).trim();
                let value = (&l[split_idx + 1 ..]).trim();
                headers.insert(name.into(), value.into());
            }
        }
        let mut body = None;
        println!("{:?}", headers);
        if headers.get("Content-Type")
            .map_or(false,
                    |ct: &String| ct.eq_ignore_ascii_case("application/x-www-form-urlencoded")) {
            let form_str = lines.next().ok_or("Expected form data")?;
            let mut payload = HashMap::new();
            for (key, val) in form_str.split("&").map(|u| {
                let sidx = u.find("=").unwrap_or(0);
                (&u[0..sidx], &u[sidx + 1..])
            }) {payload.insert(key.into(), val.into());
            }
            body = Some(RequestBody::FormUrlEncoded(payload));
        }

        //println!("{:?} {:?} {:?}", method, url, query);

        Ok(Self {
            method,
            url: url.into_boxed_slice(),
            params: query,
            headers,
            body
        })
    }
}