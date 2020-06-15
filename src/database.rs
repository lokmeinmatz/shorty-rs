use rusqlite::{Connection, params};
use std::path::Path;


pub trait Database {
    fn store_shortened(&self, long_url: String, short_url: String, hashed_ip: u32) -> Result<(), String>;
    /// get the long url and increment counter + update last visited
    fn forward(&self, short_url: String) -> Result<String, String>;

    /// same as forward, but no increment / update
    fn peek_long_url(&self, short_url: String) -> Result<String, String>;

    fn is_password(&self, pw: &str) -> bool;
}

pub struct SQLiteDB {
    connection: Connection
}

impl SQLiteDB {
    pub fn init_database<P: AsRef<Path>>(path: P) -> Result<Box<dyn Database>, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        Ok(Box::new(SQLiteDB{connection}))
    }
}


impl Database for SQLiteDB {
    fn store_shortened(&self, long_url: String, short_url: String, hashed_ip: u32) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO urls (short, long, ip_hash, created, redirects, last_redirect)\
            VALUES (?, ?, ?, datetime('now', 'localtime'), 0, datetime('now', 'localtime'))",
            params![&short_url, &long_url, hashed_ip]
        ).map(|_| ()).map_err(|e| e.to_string())
    }

    fn forward(&self, short_url: String) -> Result<String, String> {
        unimplemented!()
    }

    fn peek_long_url(&self, short_url: String) -> Result<String, String> {
        unimplemented!()
    }

    fn is_password(&self, pw: &str) -> bool {
        self.connection.prepare("SELECT * FROM passwords WHERE password = ?")
            .unwrap().query(&[pw]).unwrap().next().unwrap().is_some()
    }
}
