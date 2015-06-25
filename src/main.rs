extern crate rustc_serialize;
extern crate hyper;
extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

use std::env;
use getopts::Options;
use hyper::Client;


mod ydresponse;
mod ydclient;

use ydclient::YdClient;


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
