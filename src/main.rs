//! main module of ydcv-rs
#[macro_use]
extern crate serde_derive;

use serde_json;
use structopt;
use atty;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use env_logger;

#[cfg(any(feature="x11-clipboard", feature="clipboard2"))]
use std::time::Duration;
#[cfg(feature="x11-clipboard")]
use x11_clipboard::Clipboard;
#[cfg(feature="clipboard2")]
use clipboard2::{Clipboard, SystemClipboard};

use structopt::StructOpt;
use rustyline::Editor;
use reqwest::Client;

mod ydresponse;
mod ydclient;
mod formatters;

use crate::ydclient::YdClient;
use crate::formatters::{Formatter, PlainFormatter, AnsiFormatter, HtmlFormatter};


fn lookup_explain(client: &mut Client, word: &str, fmt: &mut dyn Formatter, raw: bool) {
    if raw {
        println!("{}", serde_json::to_string(&client.lookup_word(word, true).unwrap()).unwrap());
    } else {
        match client.lookup_word(word, false) {
            Ok(ref result) => {
                let exp = result.explain(fmt);
                fmt.print(word, &exp);
            }
            Err(err) => fmt.print(word, &format!("Error looking-up word {}: {:?}", word, err)),
        }
    }
}

#[cfg(feature="x11-clipboard")]
fn get_clipboard(clipboard: &mut Clipboard) -> String {
    clipboard
        .load(clipboard.getter.atoms.primary,
            clipboard.getter.atoms.utf8_string,
            clipboard.getter.atoms.property,
            Duration::from_secs(3))
        .map(|val| {
                String::from_utf8_lossy(&val)
                    .trim_matches('\u{0}')
                    .trim()
                    .into()
            })
        .unwrap_or_default()
}

#[cfg(feature="clipboard2")]
fn get_clipboard(clipboard: &mut SystemClipboard) -> String {
    clipboard.get_string_contents().unwrap_or_default()
}

#[derive(StructOpt)]
#[structopt(name = "ydcv", about = "A Rust version of ydcv")]
struct YdcvOptions {
    #[cfg(any(feature="x11-clipboard", feature="clipboard2"))]
    #[structopt(short = "x", long = "selection",
                help = "show explaination of current selection")]
    selection: bool,

    #[structopt(short = "H", long = "html",
                help = "HTML-style output")]
    html: bool,

    #[cfg(any(feature="notify-rust", feature="winrt-notification"))]
    #[structopt(short = "n", long = "notify",
                help = "send desktop notifications (implies -H)")]
    notify: bool,

    #[structopt(short = "r", long = "raw",
                help = "dump raw json reply from server",
                conflicts_with = "html",
                conflicts_with = "notify")]
    raw: bool,

    #[structopt(short = "c", long = "color",
                help = "[auto, always, never] use color",
                default_value = "auto")]
    color: String,

    #[cfg(feature="notify-rust")]
    #[structopt(short = "t", long = "timeout",
                help = "timeout of notification (second)",
                default_value = "30")]
    timeout: i32,

    #[structopt(value_name = "WORDS")]
    free: Vec<String>,
}


fn main() {
    env_logger::init();

    let ydcv_options = YdcvOptions::from_args();

    #[cfg(any(feature="notify-rust", feature="winrt-notification"))]
    let notify_enabled = ydcv_options.notify;
    #[cfg(not(any(feature="notify-rust", feature="winrt-notification")))]
    let notify_enabled = false;
    

    #[cfg(any(feature="x11-clipboard", feature="clipboard2"))]
    let selection_enabled = ydcv_options.selection;
    #[cfg(not(any(feature="x11-clipboard", feature="clipboard2")))]
    let selection_enabled = false;
    
    let mut client = Client::new();

    let mut html = HtmlFormatter::new(notify_enabled);
    let mut ansi = AnsiFormatter;
    let mut plain = PlainFormatter;

    #[cfg(feature="notify-rust")]
    html.set_timeout(ydcv_options.timeout * 1000);

    let fmt: &mut dyn Formatter = if ydcv_options.html || notify_enabled {
        &mut html
    } else if ydcv_options.color == "always" ||
              atty::is(atty::Stream::Stdout) && ydcv_options.color != "never" {
        &mut ansi
    } else {
        &mut plain
    };

    if ydcv_options.free.is_empty() {
        if selection_enabled {
            #[cfg(any(feature="x11-clipboard", feature="clipboard2"))]
            {
                #[cfg(feature="x11-clipboard")]
                let mut clipboard = Clipboard::new().unwrap();
                #[cfg(feature="clipboard2")]
                let mut clipboard = SystemClipboard::new().unwrap();
                let mut last = get_clipboard(&mut clipboard);
                last = last.trim().to_string();
                println!("Waiting for selection> ");
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let curr = get_clipboard(&mut clipboard).trim().to_string();
                    if curr != last {
                        last = curr.clone();
                        if !last.is_empty() {
                            lookup_explain(&mut client, &curr, fmt, ydcv_options.raw);
                            println!("Waiting for selection> ");
                        }
                    }
                }
            }
        } else {
            let mut reader = Editor::<()>::new();
            while let Ok(w) = reader.readline("> ") {
                let word = w.trim();
                reader.add_history_entry(word.as_ref());
                if !word.is_empty() {
                    lookup_explain(&mut client, &word, fmt, ydcv_options.raw);
                }
            }
        }
    } else {
        for word in ydcv_options.free {
            lookup_explain(&mut client, &word.trim(), fmt, ydcv_options.raw);
        }
    }
}
