//! parser for the returned result from YD

use ::formatters::Formatter;

/// Basic result structure
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdBasic{
    explains: Vec<String>,
    phonetic: Option<String>,
    us_phonetic: Option<String>,
    uk_phonetic: Option<String>
}

/// Web result structure
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdWeb{
    key: String,
    value: Vec<String>
}

/// Full response structure  
#[allow(non_snake_case)]
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdResponse{
    query: String,
    errorCode: i32,
    translation: Option<Vec<String>>,
    basic: Option<YdBasic>,
    web: Option<Vec<YdWeb>>
}


impl YdResponse {
    /// Explain the result in text format using a formatter
    pub fn explain(&self, fmt: &Formatter) -> String {
        let mut result: Vec<String> = vec!();

        if self.errorCode != 0 || 
            self.basic.is_none() && self.web.is_none() && self.translation.is_none(){
            result.push(fmt.red(" -- No result for this query."));
            return result.connect("\n");
        }

        if self.basic.is_none() && self.web.is_none(){
            result.push(fmt.underline(&self.query));
            result.push(fmt.cyan("  Translation:"));
            result.push("    ".to_string() + &self.translation.as_ref().unwrap().connect("；"));
            return result.connect("\n");
        }

        let phonetic = match self.basic {
            Some(ref basic) => match (basic.us_phonetic.as_ref(), basic.uk_phonetic.as_ref()) {
                (Some(us_phonetic), Some(uk_phonetic)) =>
                    format!(" UK: [{}], US: [{}]",
                        fmt.yellow(uk_phonetic),
                        fmt.yellow(us_phonetic)),
                _ => match basic.phonetic {
                        Some(ref phonetic) => format!("[{}]", fmt.yellow(&phonetic)) ,
                        _ => "".to_string()
                    }
            },
            _ => "".to_string()
        };

        result.push(format!("{} {} {}",
            fmt.underline(&self.query),
            phonetic,
            fmt.default(&self.translation.as_ref().unwrap().connect("；"))
            ));

        match self.basic {
            Some(ref basic) => if basic.explains.len() > 0 {
                result.push(fmt.cyan("  Word Explanation:"));
                for exp in &basic.explains {
                    result.push("     * ".to_string() + &exp);
                }
            },
            _ => ()
        }

        match self.web {
            Some(ref web) => {
                if web.len() > 0{
                    result.push(fmt.cyan("  Web Reference:"));
                    for item in web {
                        result.push("     * ".to_string() + &fmt.yellow(&item.key));
                        result.push("       ".to_string() + &item.value.iter()
                            .map(|x| fmt.purple(x))
                            .collect::<Vec<_>>()
                            .connect("；"));
                    }
                }
            },
            _ => ()
        }

        result.connect("\n")
    }
}

// For testing

#[cfg(test)]
use std::fmt;

#[cfg(test)]
impl fmt::Display for YdResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
         write!(f, "YdResponse('{}')", self.query)
    }
}

#[cfg(test)]
mod tests {
    use ::ydclient::*;
    use hyper::Client;
    use ::formatters::{AnsiFormatter, PlainFormatter, HtmlFormatter};

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