//! ydclient is client wrapper for Client

use std::env::var;
use std::error::Error;
use rustc_serialize::json::{ self, Json, DecoderError };
use reqwest::header::Connection;
use reqwest::Url;
use ::Client;
use super::ydresponse::YdResponse;


lazy_static! {
    /// API name
    static ref API: String = var("YDCV_API_NAME").unwrap_or_else(|_| String::from("ydcv-rs"));

    /// API key
    static ref API_KEY: String = var("YDCV_API_KEY").unwrap_or_else(|_| String::from("1323298384"));
}

/// Wrapper trait on `reqwest::Client`
pub trait YdClient{
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
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<Error>>;
    fn decode_result(&mut self, result: &str) -> Result<YdResponse, DecoderError>;
}

/// Implement wrapper client trait on `reqwest::Client`
impl YdClient for Client {

    fn decode_result(&mut self, result: &str) -> Result<YdResponse, DecoderError> {
        debug!("Recieved JSON {}", Json::from_str(result).unwrap().pretty());
        json::decode::<YdResponse>(result)
    }

    /// lookup a word on YD and returns a `YdPreponse`
    fn lookup_word(&mut self, word: &str, raw: bool) -> Result<YdResponse, Box<Error>> {
        use std::io::Read;

        let mut url = Url::parse("https://fanyi.youdao.com/openapi.do")?;
        url.query_pairs_mut().extend_pairs([
            ("keyfrom", API.as_str()),
            ("key", API_KEY.as_str()),
            ("type", "data"),
            ("doctype", "json"),
            ("version", "1.1"),
            ("q", word)
        ].into_iter());
        let mut body = String::new();
        self.get(url)
            .header(Connection::close())
            .send()?
            .read_to_string(&mut body)?;

        let raw_result = YdResponse::new_raw(body);
        if raw {
            Ok(raw_result)
        }else{
            self.decode_result(&raw_result.raw_result())
                .map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::Client;

    #[test]
    fn test_lookup_word_0(){
        assert_eq!("YdResponse('hello')",
            format!("{}", Client::new().unwrap().lookup_word("hello", false).unwrap()));
    }

    #[test]
    fn test_lookup_word_1(){
        assert_eq!("YdResponse('world')",
            format!("{}", Client::new().unwrap().lookup_word("world", false).unwrap()));
    }

    #[test]
    fn test_lookup_word_2(){
        assert_eq!("YdResponse('<+*>?_')",
            format!("{}", Client::new().unwrap().lookup_word("<+*>?_", false).unwrap()));
    }
}
