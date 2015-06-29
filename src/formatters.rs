use ansi_term::Colour::{Red, Yellow, Purple, Cyan};
use ansi_term::Style;

pub trait Formatter {
    fn red(&self, &str) -> String;
    fn yellow(&self, &str) -> String;
    fn purple(&self, &str) -> String;
    fn cyan(&self, &str) -> String;
    fn underline(&self, &str) -> String;
    fn default(&self, &str) -> String;
}

pub struct AnsiFormatter;

impl Formatter for AnsiFormatter {
	fn default(&self, s: &str) -> String { format!("{}", s) }
    fn red(&self, s: &str) -> String { format!("{}", Red.paint(s)) }
    fn yellow(&self, s: &str) -> String { format!("{}", Yellow.paint(s)) }
    fn purple(&self, s: &str) -> String { format!("{}", Purple.paint(s)) }
    fn cyan(&self, s: &str) -> String { format!("{}", Cyan.paint(s)) }

    fn underline(&self, s: &str) -> String {
    	format!("{}", Style::default().underline().paint(s))
    }
}

pub struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn red(&self, s: &str) -> String { format!(r#"<span color="red">{}</span>"#, s) }
    fn yellow(&self, s: &str) -> String { format!(r#"<span color="yellow">{}</span>"#, s) }
    fn purple(&self, s: &str) -> String { format!(r#"<span color="purple">{}</span>"#, s) }
    fn cyan(&self, s: &str) -> String { format!(r#"<span color="cyan">{}</span>"#, s) }
    fn underline(&self, s: &str) -> String { format!(r#"<span font="underline">{}</span>"#, s) }
    fn default(&self, s: &str) -> String { format!(r#"{}"#, s) }
}