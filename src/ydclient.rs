//! ydclient is client wrapper for Client

use std::io::Read;
use std::error::Error;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use hyper::Client;

use super::ydresponse::YdResponse;

/// API name from ydcv
const API: &'static str = "YouDaoCV";

/// API key from ydcv
const API_KEY: &'static str = "659600698";

/// Wrapper trait on `hypper::Client`
pub trait YdClient{
	/// lookup a word on YD and returns a `YdPreponse`
	fn lookup_word(&mut self, word: &str) -> Result<YdResponse, Box<Error>>;
}

macro_rules! try_box {
    ($expr:expr) => (match $expr {
        Ok(val) => Ok(val),
        Err(err) => Err(Box::new(err))
    })
}

impl YdClient for Client {
	fn lookup_word(&mut self, word: &str) -> Result<YdResponse, Box<Error>> {
		let url = format!("http://fanyi.youdao.com/openapi.do?\
			keyfrom={0}&key={1}&type=data&doctype=json&version=1.1&q={2}",
			API, API_KEY, word);
		let mut body = String::new();
	    try!(try!(self.get(&url).send()).read_to_string(&mut body));
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
}