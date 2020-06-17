use std::net::TcpStream;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;


/// the code of the response.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum ResponseCode {
    Ok = 200,
    MovedPermanently = 301,
    BadRequest = 400,
    NotFound = 404,
    NotAcceptable = 406
}

impl ResponseCode {
    pub fn as_reason(&self) -> &'static str {
        match self {
            ResponseCode::Ok => "Ok",
            ResponseCode::MovedPermanently => "Moved Permanently",
            ResponseCode::BadRequest => "Bad Request",
            ResponseCode::NotFound => "Not Found",
            ResponseCode::NotAcceptable => "Not Acceptable"
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ResponseBody {
    Empty,
    Html(String),
    CSS(String),
    JS(String),
    Custom{content_type: String, data: Box<[u8]>}
}

impl ResponseBody {
    /// Load a body from file and set the matching content-type, or set it to `text/plain` and load as u8 array.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        if let Some(e) = path.extension() {
            let mut ex = -1;
            if e.eq_ignore_ascii_case("html") { ex = 1; }
            else if e.eq_ignore_ascii_case("css") { ex = 2; }
            else if e.eq_ignore_ascii_case("js") { ex = 3; }
            if ex > 0 {
                let t = std::fs::read_to_string(path)?;
                match ex {
                    1 => return Ok(ResponseBody::Html(t)),
                    2 => return Ok(ResponseBody::CSS(t)),
                    3 => return Ok(ResponseBody::JS(t)),
                    _ => {}
                }
            }
        }

        let data = std::fs::read(path)?;
        Ok(ResponseBody::Custom {content_type: "text/plain".into(), data: data.into_boxed_slice()})

    }

    pub fn is_empty(&self) -> bool {
        if self == &ResponseBody::Empty { true }
        else { false }
    }

    /// returns the MIME content-type of the body (as declared)
    pub fn get_content_type(&self) -> String {
        match self {
            ResponseBody::Empty => "".into(),
            ResponseBody::Html(_) => "text/html;charset=UTF-8".into(),
            ResponseBody::CSS(_) => "text/css;charset=UTF-8".into(),
            ResponseBody::JS(_) => "text/javascript;charset=UTF-8".into(),
            ResponseBody::Custom {content_type, ..} => content_type.clone()
        }
    }

    /// returns the length in bytes
    pub fn get_length(&self) -> usize {
        match self {
            ResponseBody::Empty => 0,
            ResponseBody::Html(s) | ResponseBody::CSS(s) | ResponseBody::JS(s) => s.bytes().len(),
            ResponseBody::Custom {data, ..} => data.len()
        }
    }

    /// returns the body as byte slice
    pub fn get_bytes(&self) -> &[u8] {
        match self {
            ResponseBody::Empty => &[],
            ResponseBody::Html(s) | ResponseBody::CSS(s) | ResponseBody::JS(s) => s.as_bytes(),
            ResponseBody::Custom {data, ..} => &*data
        }
    }
}

/// The response that can be send over TCP to the client
#[derive(Debug)]
pub struct Response {
    pub code: ResponseCode,
    pub custom_headers: Option<HashMap<String, String>>,
    pub body: ResponseBody
}


impl Response {
    /// Write as http 1.1 to the TCP stream
    /// Extend here if you want to add support for Http2/3 etc
    pub fn write_html11(self, s: &mut TcpStream) -> std::io::Result<()> {
        //println!("Sending response {:?} ", self);
        writeln!(s, "HTTP/1.1 {} {}", self.code as u16, self.code.as_reason())?;

        // write custom headers
        if let Some(headers) = &self.custom_headers {
            for (key, val) in headers.iter() {
                writeln!(s, "{}: {}", key, val)?;
            }
        }
        if self.body.is_empty() { return Ok(()) }

        writeln!(s, "Content-Type: {}", self.body.get_content_type())?;
        writeln!(s, "Content-Length: {}", self.body.get_length())?;
        writeln!(s, "")?;
        // write body
        s.write_all(self.body.get_bytes())
    }
}