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


#[cfg(test)]
mod tests {
    use ::ydclient::*;
    use ::Client;
    use ::super::{AnsiFormatter, PlainFormatter, HtmlFormatter};

    #[test]
    fn test_explain_ansi(){
        assert_eq!("
\x1b[4mhello\x1b[0m [\x1b[33mhə'ləʊ; he-\x1b[0m] 你好
\x1b[36m  Word Explanation:\x1b[0m
     * n. 表示问候， 惊奇或唤起注意时的用语
     * int. 喂；哈罗
     * n. (Hello)人名；(法)埃洛
\x1b[36m  Web Reference:\x1b[0m
     * \x1b[33mHello\x1b[0m
       \x1b[35m你好\x1b[0m；\x1b[35m您好\x1b[0m；\x1b[35m哈啰\x1b[0m
     * \x1b[33mHello Kitty\x1b[0m
       \x1b[35m凯蒂猫\x1b[0m；\x1b[35m昵称\x1b[0m；\x1b[35m匿称\x1b[0m
     * \x1b[33mhello bebe\x1b[0m
       \x1b[35m哈乐哈乐\x1b[0m；\x1b[35m乐扣乐扣\x1b[0m
",format!("\n{}\n",
    Client::new()
        .lookup_word("hello").unwrap()
        .explain(&AnsiFormatter)));
    }

    #[test]
    fn test_explain_plain(){
        assert_eq!("
hello [hə'ləʊ; he-] 你好
  Word Explanation:
     * n. 表示问候， 惊奇或唤起注意时的用语
     * int. 喂；哈罗
     * n. (Hello)人名；(法)埃洛
  Web Reference:
     * Hello
       你好；您好；哈啰
     * Hello Kitty
       凯蒂猫；昵称；匿称
     * hello bebe
       哈乐哈乐；乐扣乐扣
",format!("\n{}\n",
    Client::new()
        .lookup_word("hello").unwrap()
        .explain(&PlainFormatter)));
    }

    #[test]
    fn test_explain_html_0(){
        assert_eq!(r#"
<u>hello</u> [<span color="goldenrod">hə'ləʊ; he-</span>] 你好
<span color="navy">  Word Explanation:</span>
     * n. 表示问候， 惊奇或唤起注意时的用语
     * int. 喂；哈罗
     * n. (Hello)人名；(法)埃洛
<span color="navy">  Web Reference:</span>
     * <span color="goldenrod">Hello</span>
       <span color="purple">你好</span>；<span color="purple">您好</span>；<span color="purple">哈啰</span>
     * <span color="goldenrod">Hello Kitty</span>
       <span color="purple">凯蒂猫</span>；<span color="purple">昵称</span>；<span color="purple">匿称</span>
     * <span color="goldenrod">hello bebe</span>
       <span color="purple">哈乐哈乐</span>；<span color="purple">乐扣乐扣</span>
"#,format!("\n{}\n",
    Client::new()
        .lookup_word("hello").unwrap()
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
        .lookup_word("asdakda").unwrap()
        .explain(&HtmlFormatter::new(false))));
    }

}