use ansi_term::Colour::{Red, Yellow, Purple, Cyan};
use ansi_term::Style;

pub trait Formatter {
    fn red       (&self, &str) -> String;
    fn yellow    (&self, &str) -> String;
    fn purple    (&self, &str) -> String;
    fn cyan      (&self, &str) -> String;
    fn underline (&self, &str) -> String;
    fn default   (&self, &str) -> String;
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
}

pub struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn red       (&self, s: &str) -> String { format!(r#"<span color="red">{}</span>"#, s) }
    fn yellow    (&self, s: &str) -> String { format!(r#"<span color="yellow">{}</span>"#, s) }
    fn purple    (&self, s: &str) -> String { format!(r#"<span color="purple">{}</span>"#, s) }
    fn cyan      (&self, s: &str) -> String { format!(r#"<span color="cyan">{}</span>"#, s) }
    fn underline (&self, s: &str) -> String { format!(r#"<u>{}</u>"#, s) }
    fn default   (&self, s: &str) -> String { format!(r#"{}"#, s) }
}