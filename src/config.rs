use std::io::{File, Open, Read, Write};
use rustc_serialize::Decodable;
use rustc_serialize::json::{self, Decoder, Json};

#[derive(Show, RustcEncodable, RustcDecodable)]
pub struct Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_key: String,
    pub access_secret: String
}

const PATH: &'static str = "./.cttw.conf";

pub fn read() -> Option<Config> {
    let path = Path::new(PATH);
    let mut file = match File::open_mode(&path, Open, Read) {
        Ok(f) => f,
        Err(_) => return None
    };
    let conf = Json::from_reader(&mut file).unwrap();
    Decodable::decode(&mut Decoder::new(conf)).ok()
}

pub fn write(conf: &Config) {
    let path = Path::new(PATH);
    let mut file = match File::open_mode(&path, Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("{}", e)
    };
    let _ = file.write_line(&json::encode(conf).unwrap()[]);
}
