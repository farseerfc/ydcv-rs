//! ydclient is client wrapper for Client

use std::io::Read;
use std::error::Error;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use hyper::Client;
use hyper::Url;

use super::ydresponse::YdResponse;

/// API name from ydcv
const API: &'static str = "YouDaoCV";

/// API key from ydcv
const API_KEY: &'static str = "659600698";

/// Wrapper trait on `hypper::Client`
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
    fn lookup_word(&mut self, word: &str) -> Result<YdResponse, Box<Error>>;
}

/// try and rethrow the possible error in `Box<Error>`
macro_rules! try_box {
    ($expr:expr) => (match $expr {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    })
}

/// Implement wrapper client trait on `hypper::Client`
impl YdClient for Client {

    /// lookup a word on YD and returns a `YdPreponse`
    fn lookup_word(&mut self, word: &str) -> Result<YdResponse, Box<Error>> {
        let mut url = try!(Url::parse("http://fanyi.youdao.com/openapi.do"));
        url.set_query_from_pairs(vec!(("keyfrom", API),
            ("key", API_KEY), ("type", "data"), ("doctype", "json"),
            ("version", "1.1"), ("q", word)).into_iter());

        let mut body = String::new();

        try!(
            try!(self.get(&url.serialize()).send())
            .read_to_string(&mut body));
        
        debug!("Recieved JSON {}", Json::from_str(&body).unwrap().pretty());

        try_box!(json::decode::<YdResponse>(&body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Client;

    #[test]
    fn test_lookup_word_0(){
        assert_eq!("YdResponse('hello')",
            format!("{}", Client::new().lookup_word("hello").unwrap()));
    }

    #[test]
    fn test_lookup_word_1(){
        assert_eq!("YdResponse('world')",
            format!("{}", Client::new().lookup_word("world").unwrap()));
    }

    #[test]
    fn test_lookup_word_2(){
        assert_eq!("YdResponse('<+*>?_')",
            format!("{}", Client::new().lookup_word("<+*>?_").unwrap()));
    }
}