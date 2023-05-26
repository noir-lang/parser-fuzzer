extern crate pest_based_noir_parser;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

const SQRT_FILE: &'static str = include_str!("sqrt.nr");
const SHA256_FILE: &'static str = include_str!("sha256.nr");

fn main2() -> Result<(), Error<Rule>> {
    eprintln!("{:?}", SQRT_FILE);
    let code = NoirParser::parse(Rule::start, SQRT_FILE)?.next().unwrap();
    eprintln!("{:?}", code);
    Ok(())
}

fn main3() -> Result<(), Error<Rule>> {
    eprintln!("{:?}", SHA256_FILE);
    let code = NoirParser::parse(Rule::start, SHA256_FILE)?.next().unwrap();
    eprintln!("{:?}", code);
    Ok(())
}

fn main() {
    main2().unwrap();
    main3().unwrap();
}