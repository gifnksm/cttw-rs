use std::collections::HashMap;
use oauth::{mod, Token};

mod api {
    pub const REQUEST_TOKEN: &'static str   = "https://api.twitter.com/oauth/request_token";
    pub const AUTHORIZE: &'static str       = "https://api.twitter.com/oauth/authorize";
    pub const ACCESS_TOKEN: &'static str    = "https://api.twitter.com/oauth/access_token";
    pub const STATUSES_UPDATE: &'static str = "https://api.twitter.com/1.1/statuses/update.json";
}

pub fn get_request_token(consumer: &Token) -> Token<'static> {
    let resp = oauth::get(api::REQUEST_TOKEN, consumer, None, None);
    let param = oauth::split_query(resp.as_slice());
    Token::new(param.get("oauth_token").unwrap().to_string(),
               param.get("oauth_token_secret").unwrap().to_string())
}

pub fn get_authorize_url(request: &Token) -> String {
    format!("{}?oauth_token={}", api::AUTHORIZE, request.key)
}

pub fn get_access_token(consumer: &Token, request: &Token, pin: &str) -> Token<'static> {
    let mut param = HashMap::new();
    oauth::insert_param(&mut param, "oauth_verifier", pin);
    let resp = oauth::get(api::ACCESS_TOKEN, consumer, Some(request), Some(&param));
    let param = oauth::split_query(resp.as_slice());
    Token::new(param.get("oauth_token").unwrap().to_string(),
               param.get("oauth_token_secret").unwrap().to_string())
}

pub fn tweet(consumer: &Token, access: &Token, status: &str) {
    let mut param = HashMap::new();
    oauth::insert_param(&mut param, "status", status);
    oauth::post(api::STATUSES_UPDATE, consumer, Some(access), Some(&param));
}
