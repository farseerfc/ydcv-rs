extern crate rustc_serialize;
extern crate hyper;
extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate readline;
extern crate notify_rust;
extern crate libc;

use std::env;
use libc::funcs::posix88::unistd::isatty;
use std::process::Command;
use std::thread;
use std::ffi::{CStr, CString};
use getopts::Options;
use hyper::Client;


mod ydresponse;
mod ydclient;
mod formatters;

use ydclient::YdClient;
use formatters::{Formatter, PlainFormatter, AnsiFormatter, HtmlFormatter};

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
    if let Ok(out) = Command::new("xsel").arg("-o").output() {
        if let Ok(result) = String::from_utf8(out.stdout) {
            return result;
        }
    }
    return "".to_string();
}

#[allow(dead_code)]
fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("x", "selection", "show explaination of current selection");
    opts.optflag("H", "html", "HTML-style output");
    opts.optflag("n", "notify", "send desktop notifications (implies -H)");
    opts.optopt("c", "color", "use color (auto, always, never)", "auto");

    let matches = match opts.parse(&args[1..]){
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
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
        match matches.opt_str("c") {
            Some(c) => if c == "always" || unsafe{ isatty(1) == 1} && c != "never" {
                    &mut ansi
                } else {
                    &mut plain
                },
            _ => if unsafe{ isatty(1) == 1 } {
                    &mut ansi
                } else {
                    &mut plain
                }
        }
    };

    if matches.free.len() > 0 {
        for word in matches.free {
            lookup_explain(&mut client, &word, fmt);
        }
    } else {
        if matches.opt_present("x") {
            let mut last = get_clipboard();
            println!("Waiting for selection> ");
            loop {
                thread::sleep_ms(100);
                let curr = get_clipboard();
                if curr != last {
                    last = curr.clone();
                    if last.len() > 0 {
                        lookup_explain(&mut client, &curr, fmt);
                        println!("Waiting for selection> ");
                    }
                }
            }
        } else {
            while let Ok(result) =  unsafe{ readline::readline(CStr::from_ptr(CString::new("> ").unwrap().as_ptr())) } {
                lookup_explain(&mut client, &String::from_utf8_lossy(&result.to_bytes()), fmt);
            }
        }
    }
    return;
}
