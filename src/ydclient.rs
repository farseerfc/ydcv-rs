//! ydclient is client wrapper for Client

use rand::{thread_rng, Rng};
use serde_json::{self, Error as SerdeError};
use std::env::var;
use std::error::Error;
// use reqwest::header::Connection;
use super::ydresponse::YdResponse;
use crate::lang::is_chinese;
use crate::Client;
use reqwest::Url;

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
}

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
        debug!(
            "Recieved JSON {}",
            serde_json::from_str::<YdResponse>(result)
                .and_then(|v| serde_json::to_string_pretty(&v))
                .unwrap()
        );
        serde_json::from_str(result)
    }

    /// lookup a word on YD and returns a `YdResponse`
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<dyn Error>> {
        use std::io::Read;

        let url = if let (Some(new_api_key), Some(new_app_sec)) = (NEW_API_KEY, NEW_APP_SEC) {
            let to = get_translation_lang(word);

            let salt = get_salt();
            let sign = get_sign(new_api_key, word, &salt, new_app_sec);
            api(
                "https://openapi.youdao.com/api",
                &[
                    ("appKey", new_api_key),
                    ("q", word),
                    ("from", "auto"),
                    ("to", to),
                    ("salt", &salt),
                    ("sign", &sign),
                ],
            )?
        } else {
            api(
                "https://fanyi.youdao.com/openapi.do",
                &[
                    ("keyfrom", API.as_str()),
                    ("key", API_KEY.as_str()),
                    ("type", "data"),
                    ("doctype", "json"),
                    ("version", "1.1"),
                ],
            )?
        };

        let mut body = String::new();
        self.get(url)
            // .header(Connection::close())
            .send()?
            .read_to_string(&mut body)?;

        let raw_result = YdResponse::new_raw(body.clone());
        if raw {
            raw_result.map_err(Into::into)
        } else {
            self.decode_result(&body).map_err(Into::into)
        }
    }
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
    let to = if word_is_chinese { "EN" } else { "zh-CHS" };

    to
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
