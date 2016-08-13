use super::parser::Parser;
use ::core::LispObj;

type Test<'a> = (&'a str, Vec<LispObj>);

macro_rules! tests {
    ( $( $input:expr => { $( $val:expr ),* } ),* ) => {
        let tests: Vec<Test<'static>> = vec![
            $(
                ($input, vec![ $($val),* ]) 
            ),*];
        run_tests(tests);
    }
}

fn run_tests<'a>(tests: Vec<Test<'a>>) { for (input, expected) in tests {
        let toks: Vec<_> = Parser::from_string(input, "<test>".to_string()).collect();

        println!("expected: {:?}\ntoks: {:?}", expected, toks);
        assert_eq!(toks.len(), expected.len(), "wrong number of tokens");

        for (actual, exp) in toks.into_iter().zip(expected) {
            assert!(actual.is_ok());
            let s = format!("expected: {:?}, actual: {:?}", exp, actual);
            assert!(actual.expect("Lexer should work").eq(&exp), s);
        }
    }
}

#[test]
fn test_parser_1() {
    tests!(
        "(1 2 3 4 5)" => { 
            LispObj::to_lisp_list((1..6).map(|n| int!(n)))
        },
        "hello there" => {
            symbol!("hello"),
            symbol!("there")
        }
    );
}

#[test]
fn test_parser_cons() {
    tests!(
        "(1 . 2)" => {
            cons!(int!(1), int!(2))
        },
        "(a b c . d)" => {
            cons!(symbol!("a"), cons!(symbol!("b"), cons!(symbol!("c"), symbol!("d"))))
        },
        "(1 . (2 . (3 . (4))))" => {
            LispObj::to_lisp_list((1..5).map(|n| int!(n)))
        }
    );
}
