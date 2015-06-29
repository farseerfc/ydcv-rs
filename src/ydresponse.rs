use ansi_term::Colour::{Red, Yellow, Purple, Cyan};
use ansi_term::Style;


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
	pub fn print_explain(&self) {
		if self.errorCode != 0 {
			println!("{}", Red.paint(" -- No result for this query."));
			return;
		}

		let phonetic = match self.basic {
			Some(ref basic) => if basic.us_phonetic.is_some() && basic.uk_phonetic.is_some() {
					format!(" UK: [{}], US: [{}]", 
						Yellow.paint(basic.uk_phonetic.as_ref().unwrap()),
						Yellow.paint(basic.us_phonetic.as_ref().unwrap()))
				}else{
					match basic.phonetic {
						Some(ref phonetic) => format!("[{}]", Yellow.paint(&phonetic)) ,
						None => "".to_string()
					}
				},
			None => "".to_string()
		};

		if self.basic.is_none() && self.web.is_none(){
			println!("{}", Style::default().underline().paint(&self.query));
			println!("  {}", Cyan.paint("Translation:"));
			println!("    {}", Style::default().paint(&self.translation.connect("；")));
			return;
		}

		println!("{} {} {}", 
			Style::default().underline().paint(&self.query),
			phonetic,
			Style::default().paint(&self.translation.connect("；"))
			);

		match self.basic {
			Some(ref basic) => {
				if basic.explains.len() > 0{
					println!("  {}", Cyan.paint("Word Explanation:"));
					for exp in &basic.explains {
						println!("     * {0}", Style::default().paint(&exp));
					}
				}
			},
			None => ()
		}

		match self.web {
			Some(ref web) => {
				if web.len() > 0{
					println!("  {}", Cyan.paint("Web Reference:"));
					for item in web {
						println!("     * {0}", Yellow.paint(&item.key));
						println!("       {0}", item.value.iter()
							.map(|x| Purple.paint(x).to_string())
							.collect::<Vec<_>>()
							.connect("；"));
					}
				}
			},
			None => ()
		}
	}
}



// For testing

use std::fmt;

impl fmt::Display for YdResponse {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
		 write!(f, "YdResponse('{}')", self.query)
	}
}