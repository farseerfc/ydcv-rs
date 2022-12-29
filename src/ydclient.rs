//! ydclient is client wrapper for Client

use super::ydresponse::YdResponse;
use crate::lang::is_chinese;
use log::debug;
use rand::{thread_rng, Rng};
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::{self, Error as SerdeError};
use std::env::var;
use lazy_static::lazy_static;
use std::error::Error;
use std::fmt::{self, Debug};
use std::io::Read;

const NEW_API_KEY: Option<&str> = option_env!("YD_NEW_APP_KEY");
const NEW_APP_SEC: Option<&str> = option_env!("YD_NEW_APP_SEC");

lazy_static! {
    /// API name
    static ref API: String = var("YDCV_API_NAME")
        .unwrap_or_else(|_| var("YDCV_YOUDAO_APPID")
        .unwrap_or_else(|_| String::from("ydcv-rs")));

    /// API key
    static ref API_KEY: String = var("YDCV_API_KEY")
        .unwrap_or_else(|_| var("YDCV_YOUDAO_APPSEC")
        .unwrap_or_else(|_| String::from("1323298384")));

    /// New API APPKEY in Runtime
    static ref NEW_API_KEY_RT: String = var("YD_NEW_APP_KEY")
        .unwrap_or_else(|_| String::from("ydcv-rs"));

    /// New API APPSEC in Runtime
    static ref NEW_APP_SEC_RT: String = var("YD_NEW_APP_SEC")
        .unwrap_or_else(|_| String::from("ydcv-rs"));
}

#[derive(Debug)]
enum YdClientErr {
    NewAndOldAPIError(String, String),
    NewApiValueError,
}

impl fmt::Display for YdClientErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // If both the old api and the new api fail to access, this error is returned
            YdClientErr::NewAndOldAPIError(new_api_err, old_api_err) => write!(f, "{}\n{}", old_api_err, new_api_err),
            // The error returned by not finding the variables YD_NEW_APP_KEY and YD_NEW_APP_SEC
            YdClientErr::NewApiValueError => write!(f, "New API value Error! Please make sure YD_NEW_APP_KEY and YD_NEW_APP_SEC Environment Variables is set!"),
        }
    }
}

impl Error for YdClientErr {}

/// Wrapper trait on `reqwest::Client`
pub trait YdClient {
    /// lookup a word on YD and returns a `YdPreponse`
    ///
    /// # Examples
    ///
    /// lookup "hello" and compare the result:
    ///
    /// ```
    /// assert_eq!("YdResponse('hello')",
    ///        format!("{}", Client::new().lookup_word("hello").unwrap()));
    /// ```
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<dyn Error>>;
    fn decode_result(&mut self, result: &str) -> Result<YdResponse, SerdeError>;
}

/// Implement wrapper client trait on `reqwest::Client`
impl YdClient for Client {
    fn decode_result(&mut self, result: &str) -> Result<YdResponse, SerdeError> {
        let pretty_json = serde_json::from_str::<YdResponse>(result)
            .and_then(|v| serde_json::to_string_pretty(&v));
        debug!(
            "Recieved JSON {}",
            match pretty_json {
                Ok(r) => r,
                Err(_) => result.to_owned(),
            }
        );
        serde_json::from_str(result)
    }

    /// lookup a word on YD and returns a `YdResponse`
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<dyn Error>> {
        let body = lookup_word_old_api(word, self);

        let body = if let Err(old_api_err) = body {
            let body = lookup_word_new_api(word, self);

            if let Err(new_api_err) = body {
                return Err(Box::new(YdClientErr::NewAndOldAPIError(
                    new_api_err.to_string(),
                    old_api_err.to_string(),
                )));
            }

            body
        } else {
            body
        }?;

        let raw_result = YdResponse::new_raw(body.clone());

        if raw {
            raw_result.map_err(Into::into)
        } else {
            self.decode_result(&body).map_err(Into::into)
        }
    }
}

fn lookup_word_old_api(word: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let url = api(
        "https://fanyi.youdao.com/openapi.do",
        &[
            ("keyfrom", API.as_str()),
            ("key", API_KEY.as_str()),
            ("type", "data"),
            ("doctype", "json"),
            ("version", "1.1"),
            ("q", word),
        ],
    )?;

    let mut body = String::new();
    client
        .get(url)
        // .header(Connection::close())
        .send()?
        .read_to_string(&mut body)?;

    Ok(body)
}

fn lookup_word_new_api(word: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    let (new_api_key, new_app_sec) =
        if let (Some(new_api_key), Some(new_app_sec)) = (NEW_API_KEY, NEW_APP_SEC) {
            (new_api_key, new_app_sec)
        } else if NEW_API_KEY_RT.as_str() != "ydcv-rs" && NEW_APP_SEC_RT.as_str() != "ydcv-rs" {
            (NEW_API_KEY_RT.as_str(), NEW_APP_SEC_RT.as_str())
        } else {
            return Err(Box::new(YdClientErr::NewApiValueError));
        };

    let to = get_translation_lang(word);
    let salt = get_salt();
    let sign = get_sign(new_api_key, word, &salt, new_app_sec);

    let url = api(
        "https://openapi.youdao.com/api",
        &[
            ("appKey", new_api_key),
            ("q", word),
            ("from", "auto"),
            ("to", to),
            ("salt", &salt),
            ("sign", &sign),
        ],
    )?;

    let mut body = String::new();
    client
        .get(url)
        // .header(Connection::close())
        .send()?
        .read_to_string(&mut body)?;

    Ok(body)
}

fn api(url: &str, query: &[(&str, &str)]) -> Result<Url, Box<dyn Error>> {
    let mut url = Url::parse(url)?;
    url.query_pairs_mut().extend_pairs(query.iter());

    Ok(url)
}

fn get_sign(api_key: &str, word: &str, salt: &str, app_sec: &str) -> String {
    let sign = md5::compute(format!("{}{}{}{}", api_key, word, &salt, app_sec));
    let sign = format!("{:x}", sign);

    sign
}

fn get_salt() -> String {
    let mut rng = thread_rng();
    let rand_int = rng.gen_range(1..65536);
    let salt = rand_int.to_string();

    salt
}

fn get_translation_lang(word: &str) -> &str {
    let word_is_chinese = is_chinese(word);

    if word_is_chinese {
        "EN"
    } else {
        "zh-CHS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Client;

    #[test]
    fn test_lookup_word_0() {
        assert_eq!(
            "YdResponse('hello')",
            format!("{}", Client::new().lookup_word("hello", false).unwrap())
        );
    }

    #[test]
    fn test_lookup_word_1() {
        assert_eq!(
            "YdResponse('world')",
            format!("{}", Client::new().lookup_word("world", false).unwrap())
        );
    }

    #[test]
    fn test_lookup_word_2() {
        assert_eq!(
            "YdResponse('<+*>?_')",
            format!("{}", Client::new().lookup_word("<+*>?_", false).unwrap())
        );
    }
}
