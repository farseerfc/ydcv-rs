use ::formatters::Formatter;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdBasic{
    explains: Vec<String>,
    phonetic: Option<String>,
    us_phonetic: Option<String>,
    uk_phonetic: Option<String>
}


#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdWeb{
    key: String,
    value: Vec<String>
}


#[allow(non_snake_case)]
#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct YdResponse{
    query: String,
    errorCode: i32,
    translation: Vec<String>,
    basic: Option<YdBasic>,
    web: Option<Vec<YdWeb>>
}


impl YdResponse {
    pub fn explain(&self, fmt: &Formatter) -> String {
        let mut result: Vec<String> = vec!();

        if self.errorCode != 0 {
            result.push(fmt.red(" -- No result for this query."));
            return result.connect("\n");
        }

        if self.basic.is_none() && self.web.is_none(){
            result.push(format!("{}", fmt.underline(&self.query)));
            result.push(format!("  {}", fmt.cyan("Translation:")));
            result.push(format!("    {}", fmt.default(&self.translation.connect("；"))));
            return result.connect("\n");
        }

        let phonetic = match self.basic {
            Some(ref basic) => if basic.us_phonetic.is_some() && basic.uk_phonetic.is_some() {
                    format!(" UK: [{}], US: [{}]", 
                        fmt.yellow(basic.uk_phonetic.as_ref().unwrap()),
                        fmt.yellow(basic.us_phonetic.as_ref().unwrap()))
                }else{
                    match basic.phonetic {
                        Some(ref phonetic) => format!("[{}]", fmt.yellow(&phonetic)) ,
                        None => "".to_string()
                    }
                },
            None => "".to_string()
        };

        result.push(format!("{} {} {}",
            fmt.underline(&self.query),
            phonetic,
            fmt.default(&self.translation.connect("；"))
            ));

        match self.basic {
            Some(ref basic) => {
                if basic.explains.len() > 0{
                    result.push(format!("  {}", fmt.cyan("Word Explanation:")));
                    for exp in &basic.explains {
                        result.push(format!("     * {0}", fmt.default(&exp)));
                    }
                }
            },
            None => ()
        }

        match self.web {
            Some(ref web) => {
                if web.len() > 0{
                    result.push(format!("  {}", fmt.cyan("Web Reference:")));
                    for item in web {
                        result.push(format!("     * {0}", fmt.yellow(&item.key)));
                        result.push(format!("       {0}", item.value.iter()
                            .map(|x| fmt.purple(x))
                            .collect::<Vec<_>>()
                            .connect("；")));
                    }
                }
            },
            None => ()
        }

        result.connect("\n")
    }
}

// For testing

use std::fmt;

impl fmt::Display for YdResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
         write!(f, "YdResponse('{}')", self.query)
    }
}

#[cfg(test)]
mod tests {
    use ::ydclient::*;
    use hyper::Client;

    #[test]
    fn test_explain_0(){
        assert_eq!("
\x1b[4mhello\x1b[0m [\x1b[33mhə'ləʊ; he-\x1b[0m] 你好
  \x1b[36mWord Explanation:\x1b[0m
     * n. 表示问候， 惊奇或唤起注意时的用语
     * int. 喂；哈罗
     * n. (Hello)人名；(法)埃洛
  \x1b[36mWeb Reference:\x1b[0m
     * \x1b[33mHello\x1b[0m
       \x1b[35m你好\x1b[0m；\x1b[35m您好\x1b[0m；\x1b[35m哈啰\x1b[0m
     * \x1b[33mHello Kitty\x1b[0m
       \x1b[35m凯蒂猫\x1b[0m；\x1b[35m昵称\x1b[0m；\x1b[35m匿称\x1b[0m
     * \x1b[33mhello bebe\x1b[0m
       \x1b[35m哈乐哈乐\x1b[0m；\x1b[35m乐扣乐扣\x1b[0m
",format!("\n{}\n", Client::new().lookup_word("hello").unwrap().explain()));
    }

}