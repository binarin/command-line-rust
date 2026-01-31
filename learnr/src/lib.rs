use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq)]
pub enum CLIInput {
    StdIn,
    File(String),
}

pub fn open(filename: &CLIInput) -> Result<Box<dyn BufRead>> {
    match filename {
        CLIInput::StdIn => Ok(Box::new(BufReader::new(std::io::stdin()))),
        CLIInput::File(path) => Ok(Box::new(BufReader::new(
            File::open(path).map_err(|err| anyhow!("{}: {err}", path))?,
        ))),
    }
}

impl clap::builder::ValueParserFactory for CLIInput {
    type Parser = CLIInputParser;

    fn value_parser() -> Self::Parser {
        CLIInputParser
    }
}

#[derive(Clone)]
pub struct CLIInputParser;

impl clap::builder::TypedValueParser for CLIInputParser {
    type Value = CLIInput;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        if value.eq("-") {
            Ok(CLIInput::StdIn)
        } else {
            let string_parser = clap::builder::StringValueParser::new();
            let val = string_parser.parse_ref(cmd, arg, value)?;
            Ok(CLIInput::File(val))
        }
    }
}

}
