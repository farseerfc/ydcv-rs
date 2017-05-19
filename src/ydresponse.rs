//! parser for the returned result from YD

use formatters::Formatter;

/// Basic result structure
#[derive(Serialize, Deserialize, Debug)]
pub struct YdBasic {
    explains: Vec<String>,
    phonetic: Option<String>,
    us_phonetic: Option<String>,
    uk_phonetic: Option<String>,
}

/// Web result structure
#[derive(Serialize, Deserialize, Debug)]
pub struct YdWeb {
    key: String,
    value: Vec<String>,
}

/// Full response structure
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct YdResponse {
    query: String,
    errorCode: i32,
    translation: Option<Vec<String>>,
    basic: Option<YdBasic>,
    web: Option<Vec<YdWeb>>,
}


impl YdResponse {
    pub fn new_raw(result: String) -> YdResponse {
        YdResponse {
            query: result,
            errorCode: 0,
            basic: None,
            translation: None,
            web: None,
        }
    }

    pub fn raw_result(&self) -> String {
        self.query.clone()
    }

    /// Explain the result in text format using a formatter
    pub fn explain(&self, fmt: &Formatter) -> String {
        let mut result: Vec<String> = vec![];

        if self.errorCode != 0 ||
           self.basic.is_none() && self.web.is_none() && self.translation.is_none() {
            result.push(fmt.red(" -- No result for this query."));
            return result.join("\n");
        }

        if self.basic.is_none() && self.web.is_none() {
            result.push(fmt.underline(&self.query));
            result.push(fmt.cyan("  Translation:"));
            result.push("    ".to_owned() + &self.translation.as_ref().unwrap().join("；"));
            return result.join("\n");
        }

        let phonetic = if let Some(ref basic) = self.basic {
            if let (Some(us_phonetic), Some(uk_phonetic)) =
                (basic.us_phonetic.as_ref(), basic.uk_phonetic.as_ref()) {
                format!(" UK: [{}], US: [{}]",
                        fmt.yellow(uk_phonetic),
                        fmt.yellow(us_phonetic))
            } else if let Some(ref phonetic) = basic.phonetic {
                format!("[{}]", fmt.yellow(phonetic))
            } else {
                "".to_owned()
            }
        } else {
            "".to_owned()
        };

        result.push(format!("{} {} {}",
                            fmt.underline(&self.query),
                            phonetic,
                            fmt.default(&self.translation
                                             .as_ref()
                                             .map(|v| v.join("; "))
                                             .unwrap_or_default())));

        if let Some(ref basic) = self.basic {
            if !basic.explains.is_empty() {
                result.push(fmt.cyan("  Word Explanation:"));
                for exp in &basic.explains {
                    result.push("     * ".to_owned() + exp);
                }
            }
        }

        if let Some(ref web) = self.web {
            if !web.is_empty() {
                result.push(fmt.cyan("  Web Reference:"));
                for item in web {
                    result.push("     * ".to_owned() + &fmt.yellow(&item.key));
                    result.push("       ".to_owned() +
                                &item.value
                                     .iter()
                                     .map(|x| fmt.purple(x))
                                     .collect::<Vec<_>>()
                                     .join("；"));
                }
            }
        }

        result.join("\n")
    }
}

// For testing

#[cfg(test)]
use std::fmt;

#[cfg(test)]
impl fmt::Display for YdResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "YdResponse('{}')", self.query)
    }
}
