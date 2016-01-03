//! main module of ydcv-rs
extern crate rustc_serialize;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate readline;
extern crate libc;
extern crate url;

#[cfg(feature="notify-rust")]
extern crate notify_rust;

#[cfg(feature="hyper")]
extern crate hyper;

#[cfg(feature="curl")]
extern crate curl;

use libc::isatty;

#[cfg(feature="hyper")]
pub use hyper::Client;


pub mod ydresponse;
pub mod ydclient;
pub mod formatters;

use ydclient::YdClient;
use formatters::{Formatter, PlainFormatter, AnsiFormatter, HtmlFormatter};

#[cfg(feature="curl")]
pub struct Client{
    handle: curl::http::Handle
}

#[cfg(feature="curl")]
impl Client {
    fn new() -> Client{
        Client{
            handle: curl::http::handle()
        }
    }
}

fn lookup_explain(client: &mut Client, word: &str, fmt: &mut Formatter){
    match client.lookup_word(word){
        Ok(ref result) => {
            let exp = result.explain(fmt);
            fmt.print(word, &exp);
        }
        Err(err) => fmt.print(word,
            &format!("Error looking-up word {}: {:?}", word, err))
    }
}

fn get_clipboard() -> String {
    if let Ok(out) = std::process::Command::new("xsel").arg("-o").output() {
        if let Ok(result) = String::from_utf8(out.stdout) {
            return result;
        }
    }
    return "".to_owned();
}


#[allow(dead_code)]
fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("x", "selection", "show explaination of current selection");
    opts.optflag("H", "html", "HTML-style output");
    opts.optflag("n", "notify", "send desktop notifications (implies -H)");
    opts.optopt("c", "color", "[auto, always, never] use color", "auto");

    let matches = match opts.parse(&args[1..]){
        Ok(m) => m,
        Err(f) => panic!(f.to_owned())
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] words", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut client = Client::new();

    let mut html = HtmlFormatter::new(matches.opt_present("n"));
    let mut ansi = AnsiFormatter;
    let mut plain = PlainFormatter;

    let fmt :&mut Formatter = if matches.opt_present("H") || matches.opt_present("n") {
        &mut html
    }else{
        if let Some(c) = matches.opt_str("c") {
            if c == "always" || unsafe{ isatty(1) == 1 } && c != "never" {
                &mut ansi
            } else {
                &mut plain
            }
        }else{
            if unsafe{ isatty(1) == 1 } {
                &mut ansi
            } else {
                &mut plain
            }
        }
    };

    if matches.free.is_empty() {
        if matches.opt_present("x") {
            let mut last = get_clipboard();
            println!("Waiting for selection> ");
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let curr = get_clipboard();
                if curr != last {
                    last = curr.clone();
                    if !last.is_empty() {
                        lookup_explain(&mut client, &curr, fmt);
                        println!("Waiting for selection> ");
                    }
                }
            }
        } else {
            let prompt = std::ffi::CString::new("> ").unwrap();
            while let Ok(result) = readline::readline(&prompt) {
                let word = String::from_utf8_lossy(&result.to_bytes());
                lookup_explain(&mut client, &word, fmt);
            }
        }
    } else {
        for word in matches.free {
            lookup_explain(&mut client, &word, fmt);
        }
    }
    return;
}
