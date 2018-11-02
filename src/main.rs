extern crate clap;
extern crate snapcall;

use clap::{App, Arg};
use snapcall::generate;
use std::path::Path;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1))
        .arg(Arg::with_name("filter")
            .long("filter")
            .takes_value(true))
        .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    let filter = matches.value_of("filter");
    generate(&mut std::io::stdout(), Path::new(input), filter).expect("Error");
}
