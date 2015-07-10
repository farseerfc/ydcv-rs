//! Formatters used by `YdResponse::explain`

use notify_rust::Notification;

macro_rules! def {
    ($($n:ident),*) => { $(
        fn $n (&self, s: &str) -> String;
    )* }
}

/// Base trait for formatters
pub trait Formatter {
    fn red       (&self, &str) -> String;
    fn yellow    (&self, &str) -> String;
    fn purple    (&self, &str) -> String;
    fn cyan      (&self, &str) -> String;
    fn underline (&self, &str) -> String;
    fn default   (&self, &str) -> String;

    fn print (&mut self, word: &str, body: &str);
}

/// Plain text formatter
pub struct PlainFormatter;

macro_rules! plain {
    ($($n:ident),*) => { $(
        fn $n (&self, s: &str) -> String { s.to_string() }
    )* }
}

impl Formatter for PlainFormatter {
    plain!(default, red, yellow, purple, cyan, underline);
    fn print (&mut self, _: &str, body: &str) { println!("{}", body); }
}

/// Ansi escaped colored formatter
pub struct AnsiFormatter;

macro_rules! ansi {
    ($( $n:ident = $x:expr ),*) => { $(
        fn $n (&self, s: &str) -> String {
            format!("\x1b[{}m{}\x1b[0m", $x, s)
        }
    )* }
}

impl Formatter for AnsiFormatter {
    ansi!(red=31, yellow=33, purple=35, cyan=36, underline=4);
    fn default   (&self, s: &str) -> String { s.to_string() }
    fn print (&mut self, _: &str, body: &str) { println!("{}", body); }
}

/// HTML-style formatter, suitable for desktop notification
pub struct HtmlFormatter{
    notify: bool,
    notifier: Notification,
}

impl HtmlFormatter{
    pub fn new(notify: bool) -> HtmlFormatter {
        HtmlFormatter{
            notify: notify,
            notifier: Notification::new()
        }
    }
}

macro_rules! html {
    ($( $n:ident = $x:expr ),*) => { $(
        fn $n (&self, s: &str) -> String {
            format!(r#"<span color="{}">{}</span>"#, $x, s)
        }
    )* }
}

impl Formatter for HtmlFormatter {
    html!(red="red", yellow="goldenrod", purple="purple", cyan="navy");
    fn underline (&self, s: &str) -> String { format!(r#"<u>{}</u>"#, s) }
    fn default   (&self, s: &str) -> String { format!(r#"{}"#, s) }

    fn print (&mut self, word: &str, body: &str) {
        if self.notify {
            self.notifier
                .appname("ydcv")
                .summary(word)
                .body(body)
                .timeout(30000)
                .update();
        }else{
            println!("{}", body);
        }
    }
}