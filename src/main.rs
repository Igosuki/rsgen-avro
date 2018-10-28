#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate rsgen_avro;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::stdout;
use std::process;

use docopt::Docopt;
use rsgen_avro::{Generator, Source};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = "
Usage:
  rsgen-avro (--schema=FILE | --schemas=DIR) --output=FILE [--append --add-imports -p <p>]
  rsgen-avro (-h | --help)
  rsgen-avro --version

Options:
  -h --help       Show this screen.
  --version       Show version.
  --schema=FILE   File containing an Avro schema in json format.
  --schemas=DIR   Directory containing Avro schemas in json format.
  --output=FILE   File where Rust code will be generated. Use '-' for stdout.
  -p <p>          Precision for f32/f64 default values that aren't round numbers [default: 3].
  --append        Append to output file. By default, output file is truncated.
  --add-imports   Add 'extern crate ...' at the top of the output file.
";

#[derive(Debug, Deserialize)]
struct CmdArgs {
    flag_schema: String,
    flag_schemas: String,
    flag_output: String,
    flag_p: Option<usize>,
    flag_append: bool,
    flag_add_imports: bool,
    flag_version: bool,
}

fn main() {
    let args: CmdArgs = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}", VERSION);
        process::exit(0);
    }

    let source = if &args.flag_schema != "" {
        Source::FilePath(&args.flag_schema)
    } else if &args.flag_schemas != "" {
        Source::DirPath(&args.flag_schemas)
    } else {
        eprintln!("Wrong schema source: {:?}", &args);
        process::exit(1);
    };

    let mut out: Box<Write> = if &args.flag_output == "-" {
        Box::new(stdout())
    } else {
        let mut open_opts = OpenOptions::new();
        if args.flag_append {
            open_opts.write(true).create(true).append(true)
        } else {
            open_opts.write(true).create(true).truncate(true)
        };
        Box::new(open_opts.open(&args.flag_output).unwrap_or_else(|e| {
            eprintln!("Output file error: {}", e);
            process::exit(1);
        }))
    };

    let g = Generator::builder()
        .precision(args.flag_p.unwrap_or(3))
        .add_imports(args.flag_add_imports)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Problem during prepartion: {}", e);
            process::exit(1);
        });

    g.gen(&source, &mut out).unwrap_or_else(|e| {
        eprintln!("Problem during code generation: {}", e);
        process::exit(1);
    });
}
