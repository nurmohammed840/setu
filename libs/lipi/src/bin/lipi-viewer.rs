use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = (), E = Error> = std::result::Result<T, E>;

const HELP: &str = r#"lipi-viewer — print lipi object
DESCRIPTION:
    Parses raw binary data in lipi format and prints
    a structured representation.

USAGE:
    lipi-viewer [FILE]
    cat FILE | lipi-viewer

ARGS:
    FILE        Path to a binary file
"#;

fn main() -> Result {
    let mut args = env::args().skip(1);

    if let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            print!("{HELP}");
            return Ok(());
        }
        // ---- Read from file ----
        parse_and_print_entries(fs::read(Path::new(&arg))?);
    } else {
        // ---- Read from stdin ----
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;

        if buffer.is_empty() {
            eprintln!("No input provided");
            process::exit(1);
        }

        parse_and_print_entries(buffer);
    }
    Ok(())
}

fn parse_and_print_entries(data: Vec<u8>) {
    let mut buf = &data[..];
    match lipi::Entries::parse(&mut buf) {
        Ok(entries) => println!("{entries}"),
        Err(error) => {
            let remaining = buf.len();
            let offset = data.len() - remaining;

            eprintln!(
                "\nfailed to parse; offset: {offset} (0x{offset:X}); remaining bytes: {remaining}"
            );
            eprintln!("{error:#?}");
            process::exit(1);
        }
    }
}
