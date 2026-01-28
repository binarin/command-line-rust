use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    sources: Vec<String>,
    pattern: Option<String>,
    insensitive: bool,
    seed: Option<u64>,
}


fn main() {
    println!("Hello, world!");
}
