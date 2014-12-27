use std::collections::HashMap;
use std::rand::{Rng, OsRng};
use std::str::{mod, CowString};
use rustc_serialize::base64::{mod, ToBase64};
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha1::Sha1;
use curl::http;
use time;
use url::percent_encoding;

#[deriving(Clone, Show)]
pub struct Token<'a> { pub key: CowString<'a>, pub secret: CowString<'a> }

impl<'a> Token<'a> {
    pub fn new<K, S>(key: K, secret: S) -> Token<'a>
        where K : IntoCow<'a, String, str>, S: IntoCow<'a, String, str>
    {
        Token { key: key.into_cow(), secret: secret.into_cow() }
    }
}

type ParamList<'a> = HashMap<CowString<'a>, CowString<'a>>;

pub fn insert_param<'a, K, V>(param: &mut ParamList<'a>, key: K, value: V)
    where K : IntoCow<'a, String, str>, V: IntoCow<'a, String, str>
{
    param.insert(key.into_cow(), value.into_cow());
}

pub fn join_query<'a>(param: &ParamList<'a>) -> String {
    let mut pairs = param
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k.as_slice()), encode(v.as_slice())))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.connect("&")
}

pub fn split_query<'a>(query: &'a str) -> ParamList<'a> {
    let mut param = HashMap::new();
    for q in query.split('&') {
        let mut s = q.splitn(2, '=');
        let k = s.next().unwrap();
        let v = s.next().unwrap();
        insert_param(&mut param, k, v);
    }
    param
}

fn encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::FORM_URLENCODED_ENCODE_SET)
}

fn hmac_sha1(key: &[u8], data: &[u8]) -> MacResult {
    let mut hmac = Hmac::new(Sha1::new(), key);
    hmac.input(data);
    hmac.result()
}

fn signature(method: &str, uri: &str, query: &str, consumer_secret: &str, token_secret: Option<&str>) -> String {
    let base = format!("{}&{}&{}", encode(method), encode(uri), encode(query));
    let key  = format!("{}&{}", encode(consumer_secret), encode(token_secret.unwrap_or("")));
    let conf = base64::Config {
        char_set: base64::CharacterSet::Standard,
        newline: base64::Newline::LF,
        pad: true,
        line_length: None
    };
    debug!("Signature base string: {}", base);
    debug!("Authorization header: Authorization: {}", base);
    hmac_sha1(key.as_bytes(), base.as_bytes()).code().to_base64(conf)
}

fn header(param: &ParamList) -> String{
    let mut pairs = param
        .iter()
        .filter(|&(k, _)| k.starts_with("oauth_"))
        .map(|(k, v)| format!("{}=\"{}\"", k, encode(v.as_slice())))
        .collect::<Vec<_>>();
    pairs.sort();
    format!("OAuth {}", pairs.connect(", "))
}

fn body(param: &ParamList) -> String{
    let mut pairs = param
        .iter()
        .filter(|&(k, _)| !k.starts_with("oauth_"))
        .map(|(k, v)| format!("{}={}", k, encode(v.as_slice())))
        .collect::<Vec<_>>();
    pairs.sort();
    format!("{}", pairs.connect("&"))
}

fn get_header(method: &str, uri: &str, consumer: &Token, token: Option<&Token>, other_param: Option<&ParamList>) -> (String, String) {
    let mut param = HashMap::new();
    let timestamp = format!("{}", time::now_utc().to_timespec().sec);
    let nonce = OsRng::new().unwrap().gen_ascii_chars().take(32).collect::<String>();

    insert_param(&mut param, "oauth_consumer_key",     consumer.key.as_slice());
    insert_param(&mut param, "oauth_nonce",            nonce);
    insert_param(&mut param, "oauth_signature_method", "HMAC-SHA1");
    insert_param(&mut param, "oauth_timestamp",        timestamp);
    insert_param(&mut param, "oauth_version",          "1.0");
    if let Some(tk) = token {
        insert_param(&mut param, "oauth_token", tk.key.as_slice());
    }

    if let Some(ps) = other_param {
        for (k, v) in ps.iter() {
            insert_param(&mut param, k.as_slice(), v.as_slice());
        }
    }

    let sign = signature(method, uri,
                         join_query(&param).as_slice(),
                         consumer.secret.as_slice(),
                         token.map(|t| t.secret.as_slice()));
    insert_param(&mut param, "oauth_signature", sign);

    (header(&param), body(&param))
}

pub fn get(uri: &str, consumer: &Token, token: Option<&Token>, other_param: Option<&ParamList>) -> String {
    let (header, body) = get_header("GET", uri, consumer, token, other_param);
    let resp = http::handle()
        .get(if body.len() > 0 { format!("{}?{}", uri, body) } else { format!("{}", uri) })
        .header("Authorization", header.as_slice())
        .exec().unwrap();
    debug!("{}", resp);
    assert_eq!(200, resp.get_code());
    str::from_utf8(resp.get_body()).unwrap().to_string()
}

pub fn post(uri: &str, consumer: &Token, token: Option<&Token>, other_param: Option<&ParamList>) -> String {
    let (header, body) = get_header("POST", uri, consumer, token, other_param);
    let resp = http::handle()
        .post(uri, body.as_slice())
        .header("Authorization", header.as_slice())
        .content_type("application/x-www-form-urlencoded")
        .exec().unwrap();
    debug!("{}", resp);
    assert_eq!(200, resp.get_code());
    str::from_utf8(resp.get_body()).unwrap().to_string()
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn query() {
        let mut map = HashMap::new();
        map.insert("aaa".into_cow(), "AAA".into_cow());
        map.insert("bbbb".into_cow(), "BBBB".into_cow());
        let query = super::join_query(&map);
        assert_eq!("aaa=AAA&bbbb=BBBB", query);
    }
}

