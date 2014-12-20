#![crate_type = "dylib"]

#![feature(phase)]
#![feature(plugin_registrar)]

extern crate rustc;
extern crate serialize;
extern crate crypto;
extern crate curl;
extern crate time;
extern crate url;

#[phase(plugin, link)] extern crate log;

use std::io::stdio;
use rustc::plugin::Registry;
use config::Config;
use oauth::Token;

mod config;
mod twitter;
mod oauth;

fn console_input(prompt: &str) -> String {
    print!("{}\n\t", prompt);
    let line = stdio::stdin().read_line().unwrap();
    line.trim().to_string()
}

#[plugin_registrar]
pub fn plugin_registrar(_reg: &mut Registry) {
    let conf = match config::read() {
        Some(c) => c,
        None => {
            let consumer_key    = console_input("input your consumer key:");
            let consumer_secret = console_input("input your consumer secret:");
            let consumer = Token::new(consumer_key, consumer_secret);

            let request = twitter::get_request_token(&consumer);
            println!("open the following url:");
            println!("\t{}", twitter::get_authorize_url(&request));
            let pin = console_input("input pin:");
            let access = twitter::get_access_token(&consumer, &request, pin.as_slice());

            let c = Config {
                consumer_key: consumer.key.to_string(),
                consumer_secret: consumer.secret.to_string(),
                access_key: access.key.to_string(),
                access_secret: access.secret.to_string()
            };
            config::write(&c);
            c
        }
    };

    let consumer = Token::new(conf.consumer_key, conf.consumer_secret);
    let access = Token::new(conf.access_key, conf.access_secret);

    twitter::tweet(&consumer, &access, "あなたとRust, 今すぐコンパイル");
}
