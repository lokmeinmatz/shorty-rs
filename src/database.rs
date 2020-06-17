use rusqlite::{Connection, params, NO_PARAMS};
use std::path::Path;


pub trait Database {
    fn store_shortened(&self, long_url: &str, short_url: &str, hashed_ip: u32) -> Result<(), String>;
    /// get the long url and increment counter + update last visited
    fn forward(&mut self, short_url: &str) -> Result<String, ()>;

    /// same as forward, but no increment / update
    fn peek_long_url(&self, short_url: &str) -> Result<String, ()>;

    fn is_password(&self, pw: &str) -> bool;

    fn urls_stored_last_7_days(&self, hashed_ip: u32) -> u32;
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
    fn store_shortened(&self, long_url: &str, short_url: &str, hashed_ip: u32) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO urls (short, long, ip_hash, created, redirects, last_redirect)\
            VALUES (?, ?, ?, datetime('now', 'localtime'), 0, datetime('now', 'localtime'))",
            params![short_url, long_url, hashed_ip]
        ).map(|_| ()).map_err(|e| e.to_string())
    }

    fn urls_stored_last_7_days(&self, hashed_ip: u32) -> u32 {
        self.connection.query_row(
            "SELECT Count(*) FROM urls WHERE ip_hash = ? AND created > datetime('now', 'localtime', '-1 year')", &[hashed_ip], |row| row.get(0))
            .unwrap_or(0)
    }


    fn forward(&mut self, short_url: &str) -> Result<String, ()> {
        let tx = self.connection.transaction().unwrap();
        // get url
        let long: String = tx.query_row(
            "SELECT long FROM urls WHERE short = ?", &[short_url], |row| row.get(0))
            .map_err(|_| ())?;

        tx.execute("UPDATE urls SET redirects = redirects + 1, last_redirect = datetime('now', 'localtime')", NO_PARAMS)
            .unwrap();

        tx.commit().unwrap();
        Ok(long)
    }

    fn peek_long_url(&self, short_url: &str) -> Result<String, ()> {
        self.connection.query_row(
            "SELECT long FROM urls WHERE short = ?", &[short_url], |row| row.get(0))
            .map_err(|_| ())

    }

    fn is_password(&self, pw: &str) -> bool {
        self.connection.prepare("SELECT * FROM passwords WHERE password = ?")
            .unwrap().query(&[pw]).unwrap().next().unwrap().is_some()
    }
}
