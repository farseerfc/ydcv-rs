//! parser for the returned result from YD

use crate::{formatters::Formatter, lang::is_chinese};
use scraper::{error::SelectorErrorKind, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as SerdeError, Value};

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
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YdResponse {
    query: String,
    error_code: Value,
    #[serde(flatten)]
    inner: YdResponseInner,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct YdResponseInner {
    translation: Option<Vec<String>>,
    basic: Option<YdBasic>,
    web: Option<Vec<YdWeb>>,
}

impl YdResponse {
    pub fn new_raw(result: String) -> Result<YdResponse, SerdeError> {
        serde_json::from_str(&result)
    }

    pub fn from_html(body: &str, word: &str) -> Result<YdResponse, String> {
        let html = Html::parse_document(body);
        let is_chinese = is_chinese(word);
        let res = if is_chinese {
            Self::zh2en(&html)
        } else {
            Self::en2zh(&html)
        };

        let res = match res {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        };

        Ok(YdResponse {
            query: word.to_string(),
            error_code: 0.into(),
            inner: res,
        })
    }

    /// Explain the result in text format using a formatter
    pub fn explain(&self, fmt: &dyn Formatter) -> String {
        let mut result: Vec<String> = vec![];

        let YdResponseInner {
            translation,
            basic,
            web,
        } = &self.inner;

        if self.error_code != "0" && self.error_code != 0
            || basic.is_none() && web.is_none() && translation.is_none()
        {
            result.push(fmt.red(" -- No result for this query."));
            return result.join("\n");
        }

        if basic.is_none() && web.is_none() {
            result.push(fmt.underline(&self.query));
            result.push(fmt.cyan("  Translation:"));
            result.push("    ".to_owned() + &translation.as_ref().unwrap().join("；"));
            return result.join("\n");
        }

        let phonetic = if let Some(ref basic) = basic {
            if let (Some(us_phonetic), Some(uk_phonetic)) =
                (basic.us_phonetic.as_ref(), basic.uk_phonetic.as_ref())
            {
                format!(
                    " UK: [{}], US: [{}]",
                    fmt.yellow(uk_phonetic),
                    fmt.yellow(us_phonetic)
                )
            } else if let Some(ref phonetic) = basic.phonetic {
                format!("[{}]", fmt.yellow(phonetic))
            } else {
                "".to_owned()
            }
        } else {
            "".to_owned()
        };

        result.push(format!(
            "{} {} {}",
            fmt.underline(&self.query),
            phonetic,
            fmt.default(
                &translation
                    .as_ref()
                    .map(|v| v.join("; "))
                    .unwrap_or_default()
            )
        ));

        if let Some(ref basic) = basic {
            if !basic.explains.is_empty() {
                result.push(fmt.cyan("  Word Explanation:"));
                for exp in &basic.explains {
                    result.push(fmt.default(&("     * ".to_owned() + exp)));
                }
            }
        }

        if let Some(ref web) = web {
            if !web.is_empty() {
                result.push(fmt.cyan("  Web Reference:"));
                for item in web {
                    result.push("     * ".to_owned() + &fmt.yellow(&item.key));
                    result.push(
                        "       ".to_owned()
                            + &item
                                .value
                                .iter()
                                .map(|x| fmt.purple(x))
                                .collect::<Vec<_>>()
                                .join("；"),
                    );
                }
            }
        }

        result.join("\n")
    }

    /// Lookup words by Chinese meaning.
    fn zh2en(html: &Html) -> Result<YdResponseInner, SelectorErrorKind> {
        let trans = Selector::parse(".basic .col2 .word-exp .point")?;
        let mut translations = vec![];
        html.select(&trans).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                translations.push(x.to_string());
            });
        });

        let mut explains = vec![];
        let explains_query = Selector::parse(".basic .col2 .word-exp .point")?;
        html.select(&explains_query).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                explains.push(x.to_string());
            });
        });

        let mut phonetic = String::new();
        let per_phone = Selector::parse(".phone_con .per-phone .phonetic")?;
        html.select(&per_phone).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                phonetic.push_str(x);
                return;
            });
        });

        let mut keys = vec![];
        let mut values = vec![];
        let key = Selector::parse(".web_trans .col2 .point")?;
        let value = Selector::parse(".web_trans .col2 .sen-phrase")?;
        html.select(&key).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                keys.push(x);
            });
        });
        html.select(&value).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                values.push(x.split(" ; ").map(|x| x.to_string()).collect::<Vec<_>>());
            });
        });

        let mut webs = vec![];

        for (i, c) in keys.iter().enumerate() {
            webs.push(YdWeb {
                key: c.to_string(),
                value: values[i].clone(),
            });
        }

        let resp = YdResponseInner {
            translation: Some(translations),
            basic: Some(YdBasic {
                explains,
                phonetic: Some(phonetic),
                us_phonetic: None,
                uk_phonetic: None,
            }),
            web: Some(webs),
        };

        Ok(resp)
    }

    /// Lookup words by English word.
    fn en2zh(html: &Html) -> Result<YdResponseInner, SelectorErrorKind> {
        let mut per_phone = vec![];
        let phonetic = Selector::parse(".phone_con .per-phone .phonetic")?;
        html.select(&phonetic).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                per_phone.push(x.to_string());
            });
        });

        let mut poss = vec![];
        let pos = Selector::parse(".basic .word-exp .pos")?;
        html.select(&pos).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                poss.push(x.to_string());
            });
        });

        let mut translations = vec![];
        let trans = Selector::parse(".basic .word-exp .trans")?;
        html.select(&trans).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                translations.push(x.to_string());
            });
        });

        let translations = translations
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{} {c}", poss[i]))
            .collect::<Vec<_>>();

        let mut keys = vec![];
        let mut values = vec![];
        let key = Selector::parse(".web_trans .col2 .point")?;
        let value = Selector::parse(".web_trans .col2 .sen-phrase")?;
        html.select(&key).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                keys.push(x);
            });
        });
        html.select(&value).for_each(|x| {
            x.text().into_iter().for_each(|x| {
                values.push(x.split(" ; ").map(|x| x.to_string()).collect::<Vec<_>>());
            });
        });

        let mut webs = vec![];

        for (i, c) in keys.iter().enumerate() {
            webs.push(YdWeb {
                key: c.to_string(),
                value: values[i].clone(),
            });
        }

        let resp = YdResponseInner {
            translation: None,
            basic: Some(YdBasic {
                explains: translations,
                phonetic: Some(per_phone[0].clone()),
                us_phonetic: None,
                uk_phonetic: None,
            }),
            web: Some(webs),
        };

        Ok(resp)
    }
}

// For testing

#[cfg(test)]
use std::fmt;

#[cfg(test)]
impl fmt::Display for YdResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "YdResponse('{}')", self.query)
    }
}
