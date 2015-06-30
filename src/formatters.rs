use ansi_term::Colour::{Red, Yellow, Purple, Cyan};
use ansi_term::Style;

use notify_rust::Notification;

pub trait Formatter {
    fn red       (&self, &str) -> String;
    fn yellow    (&self, &str) -> String;
    fn purple    (&self, &str) -> String;
    fn cyan      (&self, &str) -> String;
    fn underline (&self, &str) -> String;
    fn default   (&self, &str) -> String;

    fn print (&self, word: &str, body: &str);
}

pub struct AnsiFormatter;

impl Formatter for AnsiFormatter {
	fn default   (&self, s: &str) -> String { s.to_string() }
    fn red       (&self, s: &str) -> String { Red.paint(s).to_string() }
    fn yellow    (&self, s: &str) -> String { Yellow.paint(s).to_string() }
    fn purple    (&self, s: &str) -> String { Purple.paint(s).to_string() }
    fn cyan      (&self, s: &str) -> String { Cyan.paint(s).to_string() }

    fn underline (&self, s: &str) -> String {
        Style::default().underline().paint(s).to_string()
    }

    fn print (&self, _: &str, body: &str) {
        println!("{}", body);
    }
}

pub struct HtmlFormatter{
    notify: bool,
}

impl HtmlFormatter{
    pub fn new(notify: bool) -> HtmlFormatter {
        HtmlFormatter{
            notify: notify
        }
    }
}

impl Formatter for HtmlFormatter {
    fn red       (&self, s: &str) -> String { format!(r#"<span color="red">{}</span>"#, s) }
    fn yellow    (&self, s: &str) -> String { format!(r#"<span color="goldenrod">{}</span>"#, s) }
    fn purple    (&self, s: &str) -> String { format!(r#"<span color="purple">{}</span>"#, s) }
    fn cyan      (&self, s: &str) -> String { format!(r#"<span color="navy">{}</span>"#, s) }
    fn underline (&self, s: &str) -> String { format!(r#"<u>{}</u>"#, s) }
    fn default   (&self, s: &str) -> String { format!(r#"{}"#, s) }

    fn print (&self, word: &str, body: &str) {
        if self.notify {
            Notification::new()
                .appname("ydcv")
                .summary(word)
                .body(body)
                .timeout(30000)
                .show();
        }else{
            println!("{}", body);
        }
    }
}