extern crate pest_based_noir_parser;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

const SQRT_FILE: &'static str = include_str!("sqrt.nr");

#[test]
fn main() -> Result<(), Error<Rule>> {
    let code = NoirParser::parse(Rule::start, SQRT_FILE)?.next().unwrap();
    eprintln!("{:?}", code);
    eprintln!("{:?}", SQRT_FILE);
    Ok(())
}