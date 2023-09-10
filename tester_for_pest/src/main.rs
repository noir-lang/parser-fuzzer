extern crate pest_based_noir_parser;

#[macro_use]
extern crate afl;

use std::env;
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt::Write;
use std::io;
use std::io::Read;

use noirc_frontend::parse_program;

use pest::Parser;
use pest::error::Error;
use pest_based_noir_parser::{NoirParser, Rule};

fn main() -> Result<(), Error<Rule>> {
    let args: Vec<String> = env::args().collect();

    if let Some(first_arg) = args.get(1) {
        if first_arg == "--fuzz-and-save" {
            fuzz(true);
        } else if first_arg == "--parse-noir" {
            parse_noir();
        } else if first_arg == "--all" {
            if let Some(second_arg) = args.get(2) {
                return read_and_parse(&second_arg[..], true);
            } else {
                panic!("expected `--all dir`");
            }
        } else {
            return read_and_parse(&first_arg[..], false);
        }
    } else {
        fuzz(false);
    }
    Ok(())
}

fn fuzz(save: bool) {
    fuzz!(|data: &[u8]| {
        parse(data, save, true);
    });
}

fn parse(data: &[u8], save: bool, do_panic: bool) {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let data_hash = hasher.finish();
    let filename = format!("debug/{:x}", data_hash);
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
                let noirc_result = parse_program(&code[..]);
                if noirc_result.1 == vec![] {
                    writeln!(debug, "{:?}", noirc_result.0).unwrap();
                } else {
                    error = format!("noir parser failed with errors {:?}", noirc_result.1);
                }
                // assert_eq!(noirc_result.1, vec![]);
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
    if save {
        fs::write(filename, debug).unwrap();
    }
    if !error.is_empty() && do_panic {
        panic!("ERR: {}", error);
    }
}

fn read_and_parse(path: &str, all: bool) -> Result<(), Error<Rule>> {
    // traverse all files in the dir
    if all {
        let paths = fs::read_dir(path).unwrap();
    
        for maybe_file_path in paths {
            let file_path = maybe_file_path.unwrap().path();
            if file_path.file_name().unwrap().to_str().unwrap().starts_with("id") {
                let contents = fs::read(file_path)
                    .expect("Should have been able to read the file");
                parse(&contents[..], true, false);
            }
        }
        Ok(())
    } else {
        let contents = fs::read(path)
            .expect("Should have been able to read the file");
        parse(&contents[..], true, false);
        Ok(())
    }
}

fn parse_noir() {
    let mut buf = vec![];
    let stdin = io::stdin();
    let mut locked = stdin.lock();
    locked.read_to_end(&mut buf).unwrap();
    let mut error = String::new();
    let code = ::std::str::from_utf8(&buf[..]).unwrap();
    let noirc_result = parse_program(&code[..]);
    if noirc_result.1 == vec![] {
        println!("{:?}", noirc_result.0);
    } else {
        error = format!("noir parser failed with errors {:?}", noirc_result.1);
    }
    if !error.is_empty() {
        panic!("ERR: {}", error);
    }
}
