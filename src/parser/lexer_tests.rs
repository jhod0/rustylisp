use super::lexer::{Token, Lexer};
use super::lexer::Token::*;
use std::cmp::PartialEq;
use std::str;

type Test<'a> = (&'a str, Vec<Token>);

macro_rules! tests {
    ( $( $input:expr => { $( $val:expr ),* } ),* ) => {
        let tests: Vec<Test<'static>> = vec![
            $(
                ($input, vec![ $($val),* ]) 
            ),*];
        run_tests(tests);
    }
}

fn run_tests<'a>(tests: Vec<Test<'a>>) { 
    for (input, expected) in tests {
        let toks: Vec<_> = Lexer::from_string(input, "<test>".to_string()).collect();

        println!("expected: {:?}\nactual:   {:?}\n", expected, 
                 toks.iter().map(|res| {
                     match res {
                         &Ok(ref tok) => tok.tok.clone(),
                         &Err(_) => panic!("Lexer failure"),
                     }
                 }).collect::<Vec<_>>());
        assert_eq!(toks.len(), expected.len(), "wrong number of tokens");

        for (actual, exp) in toks.into_iter().zip(expected) {
            let s = format!("expected: {:?}, actual: {:?}", exp, actual.iter().map(|tok| tok.tok.clone()));
            assert!(actual.expect("Lexer should work").tok.eq(&exp), s);
        }
    }
}


#[test]
fn test_lex_1() {
    tests!(
        "(1 2 3 4)" =>  { 
            OpenParen, Number(1), Number(2), Number(3), Number(4), CloseParen 
        },
        "(hello there (testing))" => {
            OpenParen, 
            Ident("hello".to_string()), Ident("there".to_string()), 
                OpenParen,
                Ident("testing".to_string()), 
                CloseParen, 
            CloseParen
        },
        "(+123 -234 +23k)" =>  {
            OpenParen, Number(123), Number(-234), Ident("+23k".to_string()), CloseParen
        },
        "()"   => { OpenParen,  CloseParen},
        "+"    => { Ident("+".to_string()) },
        "(+ 1 - 3)" => {
            OpenParen, Ident("+".to_string()), Number(1), Ident("-".to_string()), 
            Number(3), CloseParen
        }
    );
}

#[test]
fn test_string_parsing() {
    tests!(
        "\"some string\"" => {
            QuotedString(String::from("some string"))
        },

        "(\"list\" \"of\" \"strings\")" => {
            OpenParen, 
            QuotedString(String::from("list")), QuotedString(String::from("of")),
            QuotedString(String::from("strings")), 
            CloseParen
        }
    );
}

#[test]
fn test_string_escapes() {
    tests!(
        "\"this string contains \\\"another string\\\"\"" => {
            QuotedString(String::from("this string contains \"another string\""))
        },
        "(\"list\" \"of\" \"\\\"escaped strings\\\"\")" => {
            OpenParen, 
            QuotedString(String::from("list")), QuotedString(String::from("of")),
            QuotedString(String::from("\"escaped strings\"")), 
            CloseParen
        },
        "(\"string\" \"with a \ttab\" \"with a \nnewline\"
          \"with a \\\\slash\")" => {
              OpenParen,
              QuotedString(String::from("string")), QuotedString(String::from("with a \ttab")),
              QuotedString(String::from("with a \nnewline")),
              QuotedString(String::from("with a \\slash")),
              CloseParen
        }
    );
}


#[test]
fn test_handle_special_chars() {
    tests!(
        "(look out for (#special) char'acters)" => {
            OpenParen,
            Ident(String::from("look")), Ident(String::from("out")), Ident(String::from("for")),
                OpenParen,
                    SpecialChar('#'), Ident(String::from("special")), 
                CloseParen,
            Ident(String::from("char")), SpecialChar('\''), Ident(String::from("acters")),
            CloseParen
        },

        ",`'#" => {
            SpecialChar(','), SpecialChar('`'), SpecialChar('\''), SpecialChar('#')
        },

        "' hi we are ' following special chars" => {
            SpecialChar('\''), Ident(String::from("hi")), Ident(String::from("we")), Ident(String::from("are")),
            SpecialChar('\''), Ident(String::from("following")), Ident(String::from("special")),
            Ident(String::from("chars"))
        }
    );
}
