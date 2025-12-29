use clap::{Arg, ArgAction, Command};

#[derive(Debug)]
struct Args {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> Args {
    let matches = Command::new("catr")
        .version("0.1.0")
        .author("Alexey Lebedeff <learning-rust@binarin.info>")
        .about("Rust cat")
        .arg(Arg::new("number_lines")
             .short('n')
             .long("number")
             .help("number all output lines")
             .action(ArgAction::SetTrue)
        )
        .arg(Arg::new("number_nonblank_lines")
             .short('b')
             .long("number-nonblank")
             .help("number nonempty output lines, overrides -n")
             .action(ArgAction::SetTrue)
        )
        .arg(Arg::new("files")
             .value_name("FILE")
             .help("With no FILE, or when FILE is -, read standard input.")
             .num_args(1..),
        )
        .get_matches();

    let files: Vec<String> = matches
        .get_many("files")
        .map(|it| it.cloned().collect())
        .unwrap_or(vec!["-".to_string()]);

    Args{
        files,
        number_lines: matches.get_flag("number_lines"),
        number_nonblank_lines: matches.get_flag("number_nonblank_lines"),
    }
}


fn main() {
    let args = get_args();
    dbg!(args);
}
