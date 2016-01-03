//! Formatters used by `YdResponse::explain`

#[cfg(feature="notify-rust")]
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
        fn $n (&self, s: &str) -> String { s.to_owned() }
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
    fn default   (&self, s: &str) -> String { s.to_owned() }
    fn print (&mut self, _: &str, body: &str) { println!("{}", body); }
}


/// HTML-style formatter, suitable for desktop notification
pub struct HtmlFormatter{
    notify: bool,
    #[cfg(feature="notify-rust")]
    notifier: Notification,
}

impl HtmlFormatter{
    #[cfg(feature="notify-rust")]
    pub fn new(notify: bool) -> HtmlFormatter {
        HtmlFormatter{
            notify: notify,
            notifier: Notification::new()
        }
    }

    #[cfg(not(feature="notify-rust"))]
    pub fn new(_: bool) -> HtmlFormatter {
        HtmlFormatter{
            notify: false,
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

    #[cfg(feature="notify-rust")]
    fn print (&mut self, word: &str, body: &str) {
        if self.notify {
            self.notifier
                .appname("ydcv")
                .summary(word)
                .body(body)
                .timeout(30000)
                .show().unwrap();
        }else{
            println!("{}", body);
        }
    }

    #[cfg(not(feature="notify-rust"))]
    fn print (&mut self, word: &str, body: &str) {
        println!("{}", body);
    }
}


#[cfg(test)]
mod tests {
    use ::ydclient::*;
    use ::Client;
    use ::super::{AnsiFormatter, PlainFormatter, HtmlFormatter};

    static RAW_FELIX: &'static str = r#"
    {
        "translation":["费利克斯"],
        "basic":{
            "us-phonetic":"'fi:liks",
            "phonetic":"'fi:liks",
            "uk-phonetic":"'fi:liks",
            "explains":["n. 菲力克斯（男子名）；费力克斯制导炸弹"]
        },
        "query":"Felix",
        "errorCode":0,
        "web":[
            {"value":["费利克斯","费利斯","菲力克斯"],"key":"Felix"},
            {"value":["菲利克斯·马加特","马加特","菲利斯·马加夫"],"key":"Felix Magath"},
            {"value":["费利克斯·布洛赫","布洛赫","傅里克"],"key":"Felix Bloch"}
        ]
    }"#;


    #[test]
    fn test_explain_ansi(){
        assert_eq!("
\x1b[4mFelix\x1b[0m [\x1b[33m'fi:liks\x1b[0m] 费利克斯
\x1b[36m  Word Explanation:\x1b[0m
     * n. 菲力克斯（男子名）；费力克斯制导炸弹
\x1b[36m  Web Reference:\x1b[0m
     * \x1b[33mFelix\x1b[0m
       \x1b[35m费利克斯\x1b[0m；\x1b[35m费利斯\x1b[0m；\x1b[35m菲力克斯\x1b[0m
     * \x1b[33mFelix Magath\x1b[0m
       \x1b[35m菲利克斯·马加特\x1b[0m；\x1b[35m马加特\x1b[0m；\x1b[35m菲利斯·马加夫\x1b[0m
     * \x1b[33mFelix Bloch\x1b[0m
       \x1b[35m费利克斯·布洛赫\x1b[0m；\x1b[35m布洛赫\x1b[0m；\x1b[35m傅里克\x1b[0m
",format!("\n{}\n",
    Client::new()
        .decode_result(RAW_FELIX).unwrap()
        .explain(&AnsiFormatter)));
    }

    #[test]
    fn test_explain_plain(){
        assert_eq!("
Felix ['fi:liks] 费利克斯
  Word Explanation:
     * n. 菲力克斯（男子名）；费力克斯制导炸弹
  Web Reference:
     * Felix
       费利克斯；费利斯；菲力克斯
     * Felix Magath
       菲利克斯·马加特；马加特；菲利斯·马加夫
     * Felix Bloch
       费利克斯·布洛赫；布洛赫；傅里克
",format!("\n{}\n",
    Client::new()
        .decode_result(RAW_FELIX).unwrap()
        .explain(&PlainFormatter)));
    }

    #[test]
    fn test_explain_html_0(){
        assert_eq!(r#"
<u>Felix</u> [<span color="goldenrod">'fi:liks</span>] 费利克斯
<span color="navy">  Word Explanation:</span>
     * n. 菲力克斯（男子名）；费力克斯制导炸弹
<span color="navy">  Web Reference:</span>
     * <span color="goldenrod">Felix</span>
       <span color="purple">费利克斯</span>；<span color="purple">费利斯</span>；<span color="purple">菲力克斯</span>
     * <span color="goldenrod">Felix Magath</span>
       <span color="purple">菲利克斯·马加特</span>；<span color="purple">马加特</span>；<span color="purple">菲利斯·马加夫</span>
     * <span color="goldenrod">Felix Bloch</span>
       <span color="purple">费利克斯·布洛赫</span>；<span color="purple">布洛赫</span>；<span color="purple">傅里克</span>
"#,format!("\n{}\n",
    Client::new()
        .decode_result(RAW_FELIX).unwrap()
        .explain(&HtmlFormatter::new(false))));
    }

    #[test]
    fn test_explain_html_1(){
        assert_eq!(r#"
<u>asdakda</u>
<span color="navy">  Translation:</span>
    asdakda
"#,format!("\n{}\n",
    Client::new()
        .lookup_word("asdakda", false).unwrap()
        .explain(&HtmlFormatter::new(false))));
    }

}
