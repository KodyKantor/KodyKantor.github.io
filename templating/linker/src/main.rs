/*
 * Copyright 2019 Kody Kantor
 */
extern crate getopts;

use getopts::Options;
use std::env;
use std::io::{BufReader, ErrorKind, Error};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;

static PROG: &'static str = "linker";

static LONG_DESC: &'static str = "\
This program combines a number of HTML-like files into 'real' HTML web pages.

The given input directory contains files that only contain minimal HTML. These
files are used to populate a copy of the template file. The resulting files are
written to the given output directory and should appear like a full web page.
";

/*
 * This program converts a template file into an array-like structure. Holes are
 * punched in the array where any templating directives are found. The holes
 * are then populated with the content found in the 'compiled' files.
 *
 * The resulting array is written as a complete HTML file to the output
 * directory.
 */

fn merge_template_and_html<'a>(template: &[String], mut html: File,
    merge_dest: &'a str) -> Result<(), Error> {

    let mut res = String::new();

    for line in template {
        match line.as_ref() {
            "$!post" => {
                let mut buf = String::new();
                html.read_to_string(&mut buf)?;
                res.push_str(&buf);
            },
            "$!archive" => (),
            _ => res.push_str(&format!("{}\n", line)),
        }
    }

    fs::write(merge_dest, res)
}

fn parse_template(template: &str) -> Result<Vec<String>, Error> {
    let metadata = fs::metadata(template)?;
    if !&metadata.file_type().is_file() {
        return Err(Error::new(ErrorKind::InvalidInput,
            "template must be a file"))
    }

    BufReader::new(File::open(template)?).lines().collect()
}

fn usage(opts: Options) {
    let usg = format!("{} - merge HTML templates\n\n{}", PROG, LONG_DESC);
    println!("{}", opts.usage(&usg));
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("t", "template", "template file", "TEMPLATE");
    opts.reqopt("c", "css", "template css file", "TEMPLATE");
    opts.reqopt("o", "outdir", "output directory", "OUTPUT_DIR");
    opts.reqopt("i", "indir", "directory of compiled files", "DATA_DIR");
    opts.optflag("h", "help", "print this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { usage(opts); panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        usage(opts);
        return Ok(())
    }

    let indir = matches.opt_str("indir").unwrap();
    let outdir = matches.opt_str("outdir").unwrap();
    let template = matches.opt_str("template").unwrap();
    let css = matches.opt_str("css").unwrap();

    let parsed = parse_template(&template)?;
    for dirent in fs::read_dir(indir)? {
        let file = dirent.unwrap();
        if !&file.file_type()?.is_file() {
            return Err(Error::new(ErrorKind::InvalidInput,
                "template must be a file"))
        }

        let outpath = format!("{}/{}.html", outdir,
            &file.file_name().into_string().unwrap());
        match merge_template_and_html(&parsed, File::open(&file.path())?,
            &outpath) {
            Ok(()) => (),
            Err(e) => panic!(e.to_string()),
        }
    }

    /*
     * To keep things simple we just copy the template CSS file directly to the
     * output directory.
     *
     * Append the basename of the template CSS file to the outdir to create the
     * target CSS file path.
     */
    let css_dest = Path::new(&outdir).join(Path::new(&css).file_name().unwrap());
    if let Err(e) = fs::copy(&css, &css_dest) {
        /* Let's provide a better error message if this goes south. */
        panic!("copy from {:?} to {:?} failed: {:?}",
            &css, &css_dest, e.kind());
    }

    Ok(())
}
