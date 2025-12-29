use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of cat ‘cat’
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number lines
    #[arg(short('n'), long("number"), conflicts_with("number_nonblank_lines"))]
    number_lines: bool,

    /// Number non-blank lines
    #[arg(short('b'), long("number-nonblank"))]
    number_nonblank_lines: bool,
}

fn main() {
    let args = Args::parse();
    dbg!(args);
}
