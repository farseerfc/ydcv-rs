use std::env;
use std::io::Read;

extern crate rustc_serialize;
use rustc_serialize::json;
use rustc_serialize::json::Json;

extern crate hyper;
use hyper::Client;
use hyper::header::Connection;

extern crate getopts;
use getopts::Options;

extern crate ansi_term;
use ansi_term::Colour::{Red, Yellow, Purple, Cyan};
use ansi_term::Style;


#[macro_use]
extern crate log;
extern crate env_logger;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdBasic{
	explains: Vec<String>,
	phonetic: Option<String>,
	us_phonetic: Option<String>,
	uk_phonetic: Option<String>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdWeb{
	key: String,
	value: Vec<String>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdResponse{
	query: String,
	errorCode: i32,
	translation: Vec<String>,
	basic: Option<YdBasic>,
	web: Option<Vec<YdWeb>>
}

impl YdResponse {
	fn print_explain(&self) {
		if self.errorCode != 0 {
			println!("{}", Red.paint(" -- No result for this query."));
			return;
		}

		let phonetic = match self.basic {
			Some(ref basic) => if basic.us_phonetic.is_some() && basic.uk_phonetic.is_some() {
					format!(" UK: [{}], US: [{}]", 
						Yellow.paint(&basic.uk_phonetic.clone().unwrap()),
						Yellow.paint(&basic.us_phonetic.clone().unwrap()))
				}else{
					match basic.phonetic {
						Some(ref phonetic) => format!("[{}]", Yellow.paint(&phonetic)) ,
						None => "".to_string()
					}
				},
			None => "".to_string()
		};

		if self.basic.is_none() || self.web.is_none(){
			println!("{}", Style::default().underline().paint(&self.query));
			println!("{}", Red.paint(" -- No result for this query."));
			return;
		}

		println!("{} {} {}", 
			Style::default().underline().paint(&self.query),
			phonetic,
			Style::default().paint(&self.translation.connect(""))
			);

		match self.basic {
			Some(ref basic) => {
				if basic.explains.len() > 0{
					println!("  {}", Cyan.paint("Word Explanation:"));
					for exp in &basic.explains {
						println!("     * {0}", Style::default().paint(&exp));
					}
				}
			},
			None => ()
		}

		match self.web {
			Some(ref web) => {
				if web.len() > 0{
					println!("  {}", Cyan.paint("Web Reference:"));
					for item in web {
						println!("     * {0}", Yellow.paint(&item.key));
						println!("       {0}", item.value.iter()
							.map(|x| Purple.paint(x).to_string())
							.collect::<Vec<_>>()
							.connect("；"));
					}
				}
			},
			None => ()
		}
	}
}

const API: &'static str = "YouDaoCV";
const API_KEY: &'static str = "659600698";

trait YdcvClient{
	fn lookup_word(&mut self, word: &str) -> YdResponse ;	
}

impl YdcvClient for Client {
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



fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] words", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
	env_logger::init().unwrap();

	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflag("h", "help", "print this help menu");

	let matches = match opts.parse(&args[1..]){
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};

	if matches.opt_present("h") {
		print_usage(&args[0].clone(), opts);
		return;
	}


	let mut client = Client::new();

	for word in matches.free {
	    client.lookup_word(&word).print_explain();
	}
	return;
}
