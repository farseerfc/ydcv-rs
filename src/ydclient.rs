//! ydclient is client wrapper for Client

use std::io::Read;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use hyper::Client;
use hyper::header::Connection;

use super::ydresponse::YdResponse;

/// API name from ydcv
const API: &'static str = "YouDaoCV";

/// API key from ydcv
const API_KEY: &'static str = "659600698";

/// Wrapper trait on `hypper::Client`
pub trait YdClient{
	/// lookup a word on YD and returns a `YdPreponse`
	fn lookup_word(&mut self, word: &str) -> YdResponse ;	
}


impl YdClient for Client {
	fn lookup_word(&mut self, word: &str) -> YdResponse {
		let url = format!("http://fanyi.youdao.com/openapi.do?\
			keyfrom={0}&key={1}&type=data&doctype=json&version=1.1&q={2}",
			API, API_KEY, word);

	    let mut res = self.get(&url)
		    .header(Connection::close())
		    .send().unwrap();

	    let mut body = String::new();
	    res.read_to_string(&mut body).unwrap();

	    debug!("Recieved JSON {}", Json::from_str(&body).unwrap().pretty());

	    let decoded: YdResponse = json::decode(&body).unwrap();
	    decoded
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hyper::Client;

	#[test]
	fn test_lookup_word_0(){
		assert_eq!("YdResponse('hello')",
			format!("{}", Client::new().lookup_word("hello")));
	}

	#[test]
	fn test_lookup_word_1(){
		assert_eq!("YdResponse('world')",
			format!("{}", Client::new().lookup_word("world")));
	}
}