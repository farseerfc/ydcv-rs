//! main module of ydcv-rs
extern crate rustc_serialize;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate env_logger;
extern crate getopts;
extern crate rustyline;
extern crate isatty;
extern crate reqwest;
extern crate x11_clipboard;
#[cfg(feature="notify-rust")] extern crate notify_rust;

use std::time::Duration;
use x11_clipboard::Clipboard;
use rustyline::Editor;
use isatty::stdout_isatty;
use reqwest::Client;

mod ydresponse;
mod ydclient;
mod formatters;

use ydclient::YdClient;
use formatters::{Formatter, PlainFormatter, AnsiFormatter, HtmlFormatter};


fn lookup_explain(client: &mut Client, word: &str, fmt: &mut Formatter, raw: bool) {
    if raw {
        println!("{}", client.lookup_word(word, true).unwrap().raw_result());
    } else {
        match client.lookup_word(word, false){
            Ok(ref result) => {
                let exp = result.explain(fmt);
                fmt.print(word, &exp);
            },
            Err(err) => fmt.print(word,
                &format!("Error looking-up word {}: {:?}", word, err))
        }
    }
}

fn get_clipboard(clipboard: &mut Clipboard) -> String {
    clipboard.load(
        clipboard.getter.atoms.primary,
        clipboard.getter.atoms.utf8_string,
        clipboard.getter.atoms.property,
        Duration::from_secs(3)
    )
        .map(|val| String::from_utf8_lossy(&val).trim_matches('\u{0}').trim().into())
        .unwrap_or_default()
}


fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("x", "selection", "show explaination of current selection");
    opts.optflag("H", "html", "HTML-style output");
    opts.optflag("n", "notify", "send desktop notifications (implies -H)");
    opts.optflag("r", "raw", "dump raw json reply from server");
    opts.optopt("c", "color", "[auto, always, never] use color", "auto");
    opts.optopt("t", "timeout", "timeout of notification (second)", "30");

    let matches = match opts.parse(&args[1..]){
        Ok(m) => m,
        Err(f) => panic!(f.to_owned())
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] words", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut client = Client::new().unwrap();

    let mut html = HtmlFormatter::new(matches.opt_present("n"));
    let mut ansi = AnsiFormatter;
    let mut plain = PlainFormatter;

    if let Some(t) = matches.opt_str("t") {
        let timeout: i32 = t.parse().unwrap_or(30);
        html.set_timeout(timeout * 1000);
    }

    let fmt: &mut Formatter = if matches.opt_present("H") || matches.opt_present("n") {
        &mut html
    } else if let Some(c) = matches.opt_str("c") {
        if c == "always" || stdout_isatty() && c != "never" {
            &mut ansi
        } else {
            &mut plain
        }
    } else if stdout_isatty() {
        &mut ansi
    } else {
        &mut plain
    };

    let raw = matches.opt_present("r");

    if matches.free.is_empty() {
        if matches.opt_present("x") {
            let mut clipboard = Clipboard::new().unwrap();
            let mut last = get_clipboard(&mut clipboard);
            println!("Waiting for selection> ");
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let curr = get_clipboard(&mut clipboard);
                if curr != last {
                    last = curr.clone();
                    if !last.is_empty() {
                        lookup_explain(&mut client, &curr, fmt, raw);
                        println!("Waiting for selection> ");
                    }
                }
            }
        } else {
            let mut reader = Editor::<()>::new();
            while let Ok(word) = reader.readline("> ") {
                reader.add_history_entry(&word);
                lookup_explain(&mut client, &word, fmt, raw);
            }
        }
    } else {
        for word in matches.free {
            lookup_explain(&mut client, &word, fmt, raw);
        }
    }
}
