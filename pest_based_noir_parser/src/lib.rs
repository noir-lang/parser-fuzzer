extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "../../grammar.pest"]
pub struct NoirParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let code = r#"
fn foo(x: Field) {
            for i in 0..1 {
            }
        }"#;
        let code = NoirParser::parse(Rule::start, code).unwrap().next().unwrap();
        eprintln!("{:?}", code);
    }

    #[test]
    fn test_range_expr() {
        let code =
r#"for i in 0 {
}
"#;
        let code = NoirParser::parse(Rule::for_expr, code).unwrap().next().unwrap();
        eprintln!("{:?}", code);
    }

    #[test]
    fn test_range_small() {
        let code = r#"0..(C1-1)"#;
        let code = NoirParser::parse(Rule::for_range, code).unwrap().next().unwrap();
        eprintln!("{:?}", code);
    }

    #[test]
    fn test_generate() {
        // let code = r#"0..(C1-1)"#;
        let code = NoirParser::generate("module", &[200, 200, 200, 200]);
        eprintln!("{:?}", code);
        assert_eq!(code, "    fn a  (   )  {   }    fn a  (   )  {   }    fn a  (   )  {   }");
    }

    // #[test]
    // fn test_generate2() {
    //     // let code = r#"0..(C1-1)"#;
    //     let code = NoirParser::generate("module", &[200, 200, 200, 50, 250, 50, 250, 50, 250, 100, 32, 216, 0]);
    //     eprintln!("{:?}", code);
    //     assert_eq!(code, "    fn a  (   )  {   }    fn a  (   )  {   }    fn a  (   )  {   }");
    // }
}