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
        if let Ok(code) = NoirParser::generate("program", data, Some(10_000_000)) {
            let parsed = NoirParser::parse(Rule::program, &code[..]).unwrap().next().unwrap();
        }
    });
    Ok(())
}

fn main2(file_path: &str) -> Result<(), Error<Rule>> {
    // let contents = fs::read_to_string(file_path)
    let contents = fs::read(file_path)
        .expect("Should have been able to read the file");
    let gen_result = NoirParser::generate("program", &contents[..], Some(10_000_000));
    println!("{:?}", gen_result);
    if let Ok(generated) = gen_result {
        let code = NoirParser::parse(Rule::start, &generated[..]).unwrap().next().unwrap();
        // println!("{:?}", generated);
        println!("{:?}", code);
    }
    Ok(())
}