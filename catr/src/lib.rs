use std::error::Error;
use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Alexey Lebedeff <learning-rust@binarin.info>")
        .about("Rust cat")
        .arg(Arg::with_name("number_lines")
             .short("n")
             .long("number")
             .help("number all output lines")
             .takes_value(false)
        )
        .arg(Arg::with_name("number_nonblank_lines")
             .short("b")
             .long("number-nonblank")
             .help("number nonempty output lines, overrides -n")
             .takes_value(false)
        )
        .arg(Arg::with_name("files")
             .value_name("FILE")
             .help("With no FILE, or when FILE is -, read standard input.")
             .min_values(0)
        )
        .get_matches();

    Ok(Config{
        files: matches.values_of_lossy("files").unwrap_or(vec!()),
        number_lines: matches.is_present("number_lines"),
        number_nonblank_lines: matches.is_present("number_lines"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
