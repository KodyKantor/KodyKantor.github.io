extern crate getopts;

use std::io::prelude::*;
use std::fs::{File, DirEntry};
use std::fs;
use std::io::{BufReader, Error};
use getopts::Options;
use std::env;

static PROG: &'static str = "compiler";

static LONG_DESC: &'static str = "\
This program converts plain-text files into raw HTML. The output of this program
is meant to be copied into a template file to generate a full web page.
";

/*
 * Read the given file and convert it into HTML. The output is written to a file
 * with the same name as the input file, but in the given output directory.
 */
fn convert_file(infile: &DirEntry) -> Result<String, Error> {
    let mut preformatting = false;
    let path = infile.path();
    let fname = infile.file_name().into_string().unwrap();
    let mut res: String = "".to_string();

    for line in BufReader::new(File::open(path)?).lines() {
        let lstr = line?;

        res = match lstr.get(0..2) {
            Some("$!") => {
                let mut vec: Vec<&str> = lstr.split(' ').collect();

                let val = &vec.split_off(1).join(" "); /* XXX use later */
                let key = vec[0];

                match key {
                    "$!title" | "$!title:" => {
                        /*
                         * Store the title in case we want to update the
                         * browser page title as well.
                         */
                        /*self.title = val.to_string();*/
                        format!("<h3>{}</h3>", val.to_string())
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

    }

    Ok(res)
}

fn compile<'a>(indir: &'a str, outdir: &'a str) -> std::io::Result<()> {
    match fs::read_dir(indir) {
        Err(e) => println!("{:?}", e.kind()),
        Ok(dirents) => for dirent in dirents {
            /* unwrap is okay - error would be caught previously */
            match convert_file(&dirent.unwrap()) {
                Ok(html) => /* do something */ (),
                Err(e) => println!("{:?}", e.kind()),
            }
        },
    }
    Ok(())
}

fn usage(opts: Options) {
    let usg = format!("{} - render HTML files\n\n{}", PROG, LONG_DESC);
    println!("{}", opts.usage(&usg));
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("o", "outdir", "output directory", "OUTPUT_DIR");
    opts.reqopt("i", "indir", "directory of text files", "DATA_DIR");
    opts.optflag("h", "help", "print this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { usage(opts); panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        usage(opts);
    }

    /*
     * Safe to unwrap here because getopts guarantees a value for reqopts.
     */
    let indir = matches.opt_str("indir").unwrap();
    let outdir = matches.opt_str("outdir").unwrap();

    compile(&indir, &outdir)
}
