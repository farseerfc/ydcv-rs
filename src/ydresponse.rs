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
