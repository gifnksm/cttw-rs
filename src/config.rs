use std::io::Write;
use std::fs::{File, OpenOptions};
use std::path::Path;
use rustc_serialize::Decodable;
use rustc_serialize::json::{self, Decoder, Json};

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_key: String,
    pub access_secret: String,
}

const PATH: &'static str = "./.cttw.conf";

pub fn read() -> Option<Config> {
    let path = Path::new(PATH);
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    let conf = Json::from_reader(&mut file).unwrap();
    Decodable::decode(&mut Decoder::new(conf)).ok()
}

pub fn write(conf: &Config) {
    let path = Path::new(PATH);
    let mut file = match OpenOptions::new().write(true).open(&path) {
        Ok(f) => f,
        Err(e) => panic!("{}", e),
    };
    let _ = write!(&mut file, "{}\n", &json::encode(conf).unwrap());
}
