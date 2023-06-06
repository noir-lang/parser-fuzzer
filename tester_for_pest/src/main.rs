extern crate pest_based_noir_parser;

use std::env;
use std::fs;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

fn main() -> Result<(), Error<Rule>> {
    let args: Vec<String> = env::args().collect();

    let file_path = args.get(1).expect("expected file path argument");
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    let code = NoirParser::parse(Rule::start, &contents[..])?.next().unwrap();
    println!("{:?}", code);
    Ok(())
}