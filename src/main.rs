//! main module of ydcv-rs
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate structopt_derive;
extern crate structopt;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate env_logger;
extern crate rustyline;
extern crate isatty;
extern crate reqwest;
extern crate x11_clipboard;
#[cfg(feature="notify-rust")] extern crate notify_rust;

use std::time::Duration;
use structopt::StructOpt;
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


#[derive(StructOpt)]
#[structopt(name = "ydcv", about = "A Rust version of ydcv")]
struct YdcvOptions {
    #[structopt(short = "x", long = "selection", help = "show explaination of current selection")]
    selection: bool,
    #[structopt(short = "H", long = "html", help = "HTML-style output")]
    html: bool,
    #[cfg(feature="notify-rust")]
    #[structopt(short = "n", long = "notify", help = "send desktop notifications (implies -H)")]
    notify: bool,
    #[structopt(short = "r", long = "raw", help = "dump raw json reply from server", conflicts_with = "html")]
    raw: bool,
    #[structopt(short = "c", long = "color", help = "[auto, always, never] use color", default_value = "auto")]
    color: String,
    #[cfg(feature="notify-rust")]
    #[structopt(short = "t", long = "timeout", help = "timeout of notification (second)", default_value = "30")]
    timeout: i32,
    #[structopt(value_name = "WORDS")]
    free: Vec<String>
}


fn main() {
    env_logger::init().unwrap();

    let ydcv_options = YdcvOptions::from_args();

    #[cfg(feature="notify-rust")]
    let notify_enabled = ydcv_options.notify;
    #[cfg(not(feature="notify-rust"))]
    let notify_enabled = false;

    let mut client = Client::new().unwrap();

    let mut html = HtmlFormatter::new(notify_enabled);
    let mut ansi = AnsiFormatter;
    let mut plain = PlainFormatter;

    #[cfg(feature="notify-rust")]
    html.set_timeout(ydcv_options.timeout * 1000);

    let fmt: &mut Formatter = if ydcv_options.html || notify_enabled {
        &mut html
    } else if ydcv_options.color == "always" || stdout_isatty() && ydcv_options.color != "never" {
        &mut ansi
    } else {
        &mut plain
    };

    if ydcv_options.free.is_empty() {
        if ydcv_options.selection {
            let mut clipboard = Clipboard::new().unwrap();
            let mut last = get_clipboard(&mut clipboard);
            println!("Waiting for selection> ");
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let curr = get_clipboard(&mut clipboard);
                if curr != last {
                    last = curr.clone();
                    if !last.is_empty() {
                        lookup_explain(&mut client, &curr, fmt, ydcv_options.raw);
                        println!("Waiting for selection> ");
                    }
                }
            }
        } else {
            let mut reader = Editor::<()>::new();
            while let Ok(word) = reader.readline("> ") {
                reader.add_history_entry(&word);
                lookup_explain(&mut client, &word, fmt, ydcv_options.raw);
            }
        }
    } else {
        for word in ydcv_options.free {
            lookup_explain(&mut client, &word, fmt, ydcv_options.raw);
        }
    }
}
