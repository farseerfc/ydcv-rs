extern crate rustc_serialize;
extern crate hyper;
extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate linenoise;

use std::env;
use std::process::Command;
use std::thread;
use getopts::Options;
use hyper::Client;


mod ydresponse;
mod ydclient;
mod formatters;

use ydclient::YdClient;
use formatters::{Formatter, AnsiFormatter, HtmlFormatter};

fn lookup_explain(client: &mut Client, word: &String, fmt: &Formatter){
    match client.lookup_word(&word){
        Ok(ref result) =>  println!("{}", result.explain(fmt)),
        Err(err) => println!("Error during lookup word {}: {:?}", word, err)
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
    opts.optflag("H", "HTML", "HTML output");

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

    let html = HtmlFormatter;
    let ansi = AnsiFormatter;

    let fmt :&Formatter = if matches.opt_present("H") { &html }else{ &ansi };

    if matches.free.len() > 0 {
        for word in matches.free {
            lookup_explain(&mut client, &word, fmt);
        }
    } else {
        if matches.opt_present("x") {
            let mut last = get_clipboard();
            print!("Waiting for selection> ");
            loop {
                thread::sleep_ms(100);
                let curr = get_clipboard();
                if curr != last {
                    last = curr.clone();
                    if last.len() > 0 {
                        lookup_explain(&mut client, &curr, fmt);
                        print!("Waiting for selection> ");
                    }
                }
            }
        } else {
            while let Some(word) =  linenoise::input("> ") {
                lookup_explain(&mut client, &word, fmt);
                linenoise::history_add(&word);
            }
        }
    }
    return;
}
