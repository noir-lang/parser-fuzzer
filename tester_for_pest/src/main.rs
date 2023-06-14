extern crate pest_based_noir_parser;

#[macro_use]
extern crate afl;

use std::env;
use std::fs;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

fn main() -> Result<(), Error<Rule>> {
    let args: Vec<String> = env::args().collect();

    if let Some(file_path) = args.get(1) {
        return main2(&file_path[..]);
    }
    fuzz!(|data: &[u8]| {
        let code = NoirParser::generate("program", data);
        let parsed = NoirParser::parse(Rule::program, &code[..]).unwrap().next().unwrap();
    });
    Ok(())
}

fn main2(file_path: &str) -> Result<(), Error<Rule>> {
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    let code = NoirParser::parse(Rule::start, &contents[..])?.next().unwrap();
    println!("{:?}", code);
    Ok(())
}