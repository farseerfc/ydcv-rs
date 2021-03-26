//! Formatters used by `YdResponse::explain`

#[cfg(all(feature = "notify", unix))]
use notify_rust::Notification;

#[cfg(all(feature = "notify", windows))]
use winrt_notification::{Duration, Toast};

extern crate htmlescape;
use htmlescape::encode_minimal;

macro_rules! def {
    ($($n:ident),*) => { $(
        fn $n (&self, s: &str) -> String;
    )* }
}

/// Base trait for formatters
pub trait Formatter {
    def!(red);
    def!(yellow);
    def!(purple);
    def!(cyan);
    def!(underline);
    def!(default);

    fn print(&mut self, word: &str, body: &str);
}

/// Plain text formatter
pub struct PlainFormatter;

macro_rules! plain {
    ($($n:ident),*) => { $(
        fn $n (&self, s: &str) -> String { s.to_owned() }
    )* }
}

impl PlainFormatter {
    pub fn new(_: bool) -> PlainFormatter {
        PlainFormatter {}
    }
}

impl Formatter for PlainFormatter {
    plain!(default, red, yellow, purple, cyan, underline);

    fn print(&mut self, _: &str, body: &str) {
        println!("{}", body);
    }
}

/// WinFormatter text formatter

#[cfg(all(feature = "notify", windows))]
pub struct WinFormatter {
    notify: bool,
}

#[cfg(all(feature = "notify", windows))]
impl WinFormatter {
    pub fn new(notify: bool) -> WinFormatter {
        WinFormatter { notify }
    }
}

#[cfg(all(feature = "notify", windows))]
macro_rules! ignore {
    ($($n:ident),*) => { $(
        fn $n (&self, _s: &str) -> String { "".to_owned() }
    )* }
}

#[cfg(all(feature = "notify", windows))]
impl Formatter for WinFormatter {
    plain!(default, red, yellow, purple, underline);
    ignore!(cyan);

    fn print(&mut self, _word: &str, body: &str) {
        if self.notify {
            // windows notification has limited lines
            // so we display as little as possible
            let lines: Vec<&str> = body.split('\n').filter(|x| x.len() > 0).collect();
            Toast::new(Toast::POWERSHELL_APP_ID)
                .title(lines[0])
                .text1(&lines[1..].join("\n"))
                .duration(Duration::Long)
                .show()
                .expect("ydcv: unable to toast");
        } else {
            println!("{}", body);
        }
    }
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

impl AnsiFormatter {
    pub fn new(_: bool) -> AnsiFormatter {
        AnsiFormatter {}
    }
}

impl Formatter for AnsiFormatter {
    ansi!(red = 31, yellow = 33, purple = 35, cyan = 36, underline = 4);

    fn default(&self, s: &str) -> String {
        s.to_owned()
    }

    fn print(&mut self, _: &str, body: &str) {
        println!("{}", body);
    }
}

/// HTML-style formatter, suitable for desktop notification
#[cfg(all(feature = "notify", unix))]
pub struct HtmlFormatter {
    notify: bool,
    notifier: Notification,
    timeout: i32,
}

#[cfg(not(all(feature = "notify", unix)))]
pub struct HtmlFormatter;

impl HtmlFormatter {
    #[cfg(all(feature = "notify", unix))]
    pub fn new(notify: bool) -> HtmlFormatter {
        HtmlFormatter {
            notify,
            notifier: Notification::new(),
            timeout: 30000,
        }
    }

    #[cfg(not(all(feature = "notify", unix)))]
    pub fn new(_: bool) -> HtmlFormatter {
        HtmlFormatter {}
    }

    #[cfg(all(feature = "notify", unix))]
    pub fn set_timeout(&mut self, timeout: i32) {
        self.timeout = timeout;
    }
}

macro_rules! html {
    ($( $n:ident = $x:expr ),*) => { $(
        fn $n (&self, s: &str) -> String {
            format!(r#"<span color="{}">{}</span>"#, $x, encode_minimal(s))
        }
    )* }
}

impl Formatter for HtmlFormatter {
    html!(
        red = "red",
        yellow = "goldenrod",
        purple = "purple",
        cyan = "navy"
    );
    fn underline(&self, s: &str) -> String {
        format!(r#"<u>{}</u>"#, encode_minimal(s))
    }
    fn default(&self, s: &str) -> String {
        encode_minimal(s)
    }

    #[cfg(all(feature = "notify", unix))]
    fn print(&mut self, word: &str, body: &str) {
        if self.notify {
            self.notifier
                .appname("ydcv")
                .summary(word)
                .body(body)
                .timeout(self.timeout)
                .show()
                .unwrap();
        } else {
            println!("{}", body);
        }
    }

    #[cfg(not(all(feature = "notify", unix)))]
    fn print(&mut self, _: &str, body: &str) {
        println!("{}", body);
    }
}

#[cfg(test)]
mod tests {
    use crate::formatters::{AnsiFormatter, HtmlFormatter, PlainFormatter};
    use crate::ydclient::*;
    use reqwest::blocking::Client;

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
    fn test_explain_ansi() {
        let result = format!(
            "\n{}\n",
            Client::new()
                .decode_result(RAW_FELIX)
                .unwrap()
                .explain(&AnsiFormatter::new(false))
        );
        assert_eq!(
            "
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
",
            result
        );
    }

    #[test]
    fn test_explain_plain() {
        let result = format!(
            "\n{}\n",
            Client::new()
                .decode_result(RAW_FELIX)
                .unwrap()
                .explain(&PlainFormatter::new(false))
        );
        assert_eq!(
            "
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
",
            result
        );
    }

    #[test]
    fn test_explain_html_0() {
        assert_eq!(
            r#"
<u>Felix</u> [<span color="goldenrod">&#x27;fi:liks</span>] 费利克斯
<span color="navy">  Word Explanation:</span>
     * n. 菲力克斯（男子名）；费力克斯制导炸弹
<span color="navy">  Web Reference:</span>
     * <span color="goldenrod">Felix</span>
       <span color="purple">费利克斯</span>；<span color="purple">费利斯</span>；<span color="purple">菲力克斯</span>
     * <span color="goldenrod">Felix Magath</span>
       <span color="purple">菲利克斯·马加特</span>；<span color="purple">马加特</span>；<span color="purple">菲利斯·马加夫</span>
     * <span color="goldenrod">Felix Bloch</span>
       <span color="purple">费利克斯·布洛赫</span>；<span color="purple">布洛赫</span>；<span color="purple">傅里克</span>
"#,
            format!(
                "\n{}\n",
                Client::new()
                    .decode_result(RAW_FELIX)
                    .unwrap()
                    .explain(&HtmlFormatter::new(false))
            )
        );
    }

    #[test]
    fn test_explain_html_1() {
        let result = format!(
            "\n{}\n",
            Client::new()
                .lookup_word("asdakda", false)
                .unwrap()
                .explain(&HtmlFormatter::new(false))
        );
        assert_eq!(
            r#"
<u>asdakda</u>
<span color="navy">  Translation:</span>
    asdakda
"#,
            result
        );
    }

    #[test]
    fn test_explain_html_2() {
        let result = format!(
            "\n{}\n",
            Client::new()
                .lookup_word("comment", false)
                .unwrap()
                .explain(&HtmlFormatter::new(false))
        );
        assert_eq!(
            r#"
<u>comment</u> [<span color="goldenrod">ˈkɒment</span>] 评论
<span color="navy">  Word Explanation:</span>
     * n. 评论，意见；批评，指责；说明，写照；&lt;旧&gt;解说，注释；（计算机）注解
     * v. 评论，发表意见；（计算机）注解，把（部分程序）转成注解
     * 【名】 （Comment）（美、瑞、法）科门特（人名）
<span color="navy">  Web Reference:</span>
     * <span color="goldenrod">Comment</span>
       <span color="purple">注释</span>；<span color="purple">注解</span>；<span color="purple">客户点评</span>
     * <span color="goldenrod">No Comment</span>
       <span color="purple">不予置评</span>；<span color="purple">无可奉告</span>；<span color="purple">不予回答</span>；<span color="purple">无意见</span>
     * <span color="goldenrod">Fair comment</span>
       <span color="purple">公正评论</span>；<span color="purple">公允评论</span>；<span color="purple">合理评论</span>；<span color="purple">公正的评论</span>
"#,
            result
        );
    }
}
