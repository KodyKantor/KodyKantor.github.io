use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;

fn main() -> std::io::Result<()> {
    let comp = Compiler::new("../2019/pg_temp_files");
    comp.compile()
}

pub struct Compiler<'a> {
    file: &'a str,
    title: String, 
}

impl Compiler<'_> {
    pub fn new(file: &str) -> Compiler {
        Compiler {
            file: file,
            title: "".to_string(), 
        }
    }

    pub fn compile(mut self) -> std::io::Result<()> {
        let mut preformatting = false;
        for line in BufReader::new(File::open(self.file)?).lines() {
            let lstr = String::from(line?);

            let res = match lstr.get(0..2) {
                Some("$!") => {
                    let mut vec: Vec<&str> = lstr.split(" ").collect();

                    let val = &vec.split_off(1).join(" ");
                    let key = vec[0];

                    match key {
                        "$!title" | "$!title:" => {
                            /*
                             * Store the title in case we want to update the
                             * browser page title as well.
                             */
                            self.title = val.to_string();
                            format!("<h3>{}</h3>", val)
                        },
                        "$!date" | "$!date:" => "".to_string(), /* XXX */
                        "$!categories" | "$!categories:" => "".to_string(),
                        _ => { "".to_string() },
                    }
                },
                _ => match lstr.as_ref() {
                    "" => {
                        if !preformatting {
                            "<br /><br />".to_string()
                        } else {
                            "".to_string()
                        }
                    },
                    "```" => {
                        if !preformatting {
                            preformatting = true;
                            "<pre>".to_string()
                        } else {
                            preformatting = false;
                            "</pre>".to_string()
                        }
                    },
                    _ => lstr.to_string(),
                },
            };

            println!("{}", res);
        }
        Ok(())
    }
}
