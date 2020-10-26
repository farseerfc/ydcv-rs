//! main module of ydcv-rs
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
#[cfg(feature = "clipboard")]
use clipboard2::{Clipboard, SystemClipboard};

#[cfg(unix)]
#[cfg(feature = "clipboard")]
use x11_clipboard::Clipboard;

use reqwest::{Client, ClientBuilder};
use rustyline::Editor;
use structopt::StructOpt;

mod formatters;
mod ydclient;
mod ydresponse;

#[cfg(windows)]
#[cfg(feature = "notify")]
use crate::formatters::WinFormatter;
use crate::formatters::{AnsiFormatter, Formatter, HtmlFormatter, PlainFormatter};
use crate::ydclient::YdClient;

fn lookup_explain(client: &mut Client, word: &str, fmt: &mut dyn Formatter, raw: bool) {
    if raw {
        println!(
            "{}",
            serde_json::to_string(&client.lookup_word(word, true).unwrap()).unwrap()
        );
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

#[cfg(windows)]
#[cfg(feature = "clipboard")]
fn get_clipboard(clipboard: &mut SystemClipboard) -> String {
    clipboard.get_string_contents().unwrap_or_default()
}

#[derive(StructOpt)]
#[structopt(name = "ydcv", about = "A Rust version of ydcv")]
struct YdcvOptions {
    #[cfg(feature = "clipboard")]
    #[structopt(
        short = "x",
        long = "selection",
        help = "show explaination of current selection"
    )]
    selection: bool,

    #[cfg(windows)]
    #[cfg(feature = "clipboard")]
    #[structopt(
        short = "i",
        long = "interval",
        help = "time interval between selection in msec (default: 1000 on windows and 0 on others)",
        default_value = "1000"
    )]
    interval: u64,

    #[cfg(unix)]
    #[cfg(feature = "clipboard")]
    #[structopt(
        short = "i",
        long = "interval",
        help = "time interval between selection in msec (default: 1000 on windows and 0 on others)",
        default_value = "0"
    )]
    interval: u64,

    #[structopt(short = "H", long = "html", help = "HTML-style output")]
    html: bool,

    #[cfg(feature = "notify")]
    #[structopt(
        short = "n",
        long = "notify",
        help = "send desktop notifications (implies -H on X11)"
    )]
    notify: bool,

    #[structopt(
        short = "r",
        long = "raw",
        help = "dump raw json reply from server",
        conflicts_with = "html",
        conflicts_with = "notify"
    )]
    raw: bool,

    #[structopt(
        short = "c",
        long = "color",
        help = "[auto, always, never] use color",
        default_value = "auto"
    )]
    color: String,

    #[cfg(unix)]
    #[cfg(feature = "notify")]
    #[structopt(
        short = "t",
        long = "timeout",
        help = "timeout of notification (second)",
        default_value = "30"
    )]
    timeout: i32,

    #[structopt(value_name = "WORDS")]
    free: Vec<String>,
}

fn main() {
    env_logger::init();

    let ydcv_options = YdcvOptions::from_args();

    #[cfg(feature = "notify")]
    let notify_enabled = ydcv_options.notify;
    #[cfg(not(feature = "notify"))]
    let notify_enabled = false;

    #[cfg(feature = "clipboard")]
    let selection_enabled = ydcv_options.selection;

    #[cfg(feature = "clipboard")]
    let interval = ydcv_options.interval;

    #[cfg(not(feature = "clipboard"))]
    let selection_enabled = false;

    let mut client = ClientBuilder::new().use_sys_proxy().build().unwrap();

    let mut html = HtmlFormatter::new(notify_enabled);
    let mut ansi = AnsiFormatter::new(notify_enabled);
    let mut plain = PlainFormatter::new(notify_enabled);
    #[cfg(windows)]
    #[cfg(feature = "notify")]
    let mut win = WinFormatter::new(notify_enabled);

    #[cfg(unix)]
    #[cfg(feature = "notify")]
    html.set_timeout(ydcv_options.timeout * 1000);

    let fmt: &mut dyn Formatter =
        if ydcv_options.html || (notify_enabled && cfg!(unix) && cfg!(feature = "notify")) {
            &mut html
        } else if notify_enabled {
            #[cfg(all(windows, feature = "notify"))]
            {
                &mut win
            }
            #[cfg(not(all(windows, feature = "notify")))]
            {
                &mut plain
            }
        } else if ydcv_options.color == "always"
            || atty::is(atty::Stream::Stdout) && ydcv_options.color != "never"
        {
            &mut ansi
        } else {
            &mut plain
        };

    if ydcv_options.free.is_empty() {
        if selection_enabled {
            #[cfg(unix)]
            #[cfg(feature = "clipboard")]
            {
                let clipboard = Clipboard::new().unwrap();
                let mut last = String::new();

                println!("Waiting for selection> ");

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(interval));
                    if let Ok(curr) = clipboard.load_wait(
                        clipboard.getter.atoms.primary,
                        clipboard.getter.atoms.utf8_string,
                        clipboard.getter.atoms.property,
                    ) {
                        let curr = String::from_utf8_lossy(&curr);
                        let curr = curr.trim_matches('\u{0}').trim();
                        if !curr.is_empty() && last != curr {
                            last = curr.to_owned();
                            lookup_explain(&mut client, curr, fmt, ydcv_options.raw);
                            println!("Waiting for selection> ");
                        }
                    }
                }
            }

            #[cfg(windows)]
            #[cfg(feature = "clipboard")]
            {
                let mut clipboard = SystemClipboard::new().unwrap();
                let mut last = get_clipboard(&mut clipboard);
                last = last.trim().to_string();
                println!("Waiting for selection> ");
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(interval));
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
                reader.add_history_entry(word);
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
