extern crate pest_based_noir_parser;

#[macro_use]
extern crate afl;

use std::env;
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt::Write;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

fn main() -> Result<(), Error<Rule>> {
    let args: Vec<String> = env::args().collect();

    if let Some(file_path) = args.get(1) {
        return main2(&file_path[..]);
    }
    fuzz!(|data: &[u8]| {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let data_hash = hasher.finish();
        let filename = format!("{:x}", data_hash);
        let mut debug = String::new();
        let mut error = String::new();
        let program_code = NoirParser::generate("program", data, Some(10_000_000));
        //
        if let Ok(code) = program_code {
            writeln!(debug, "{}", code).unwrap();
            let parsed = NoirParser::parse(Rule::program, &code[..]);
            if let Ok(mut foo) = parsed {
                if let Some(bar) = foo.next() {
                    writeln!(debug, "{:?}", bar).unwrap();
                } else {
                    error = "second unwrap failed".to_string();
                }
            } else {
                error = "first unwrap failed".to_string();
            }
        } else {
            error = "generation exceeded the limit".to_string();
        }
        if !error.is_empty() {
            writeln!(debug, "ERR: {}", error).unwrap();
        }
        fs::write(filename, debug).unwrap();
        if !error.is_empty() {
            panic!("ERR: {}", error);
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