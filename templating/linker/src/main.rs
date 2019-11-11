extern crate getopts;

use getopts::Options;
use std::env;

static PROG: &'static str = "linker";

static LONG_DESC: &'static str = "\
This program combines a number of HTML-like files into 'real' HTML web pages.

The given input directory contains files that only contain minimal HTML. These
files are used to populate a copy of the template file. The resulting files are
written to the given output directory and should appear like a full web page.
";

fn usage(opts: Options) {
    let usg = format!("{} - merge HTML templates\n\n{}", PROG, LONG_DESC);
    println!("{}", opts.usage(&usg));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.reqopt("t", "template", "template file", "TEMPLATE");
    opts.reqopt("o", "outdir", "output directory", "OUTPUT_DIR");
    opts.reqopt("i", "indir", "directory of compiled files", "DATA_DIR");
    opts.optflag("h", "help", "print this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { usage(opts); panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        usage(opts);
        return;
    }

    let indir = matches.opt_str("indir");
    let outdir = matches.opt_str("outdir");
    let template = matches.opt_str("template");
}
