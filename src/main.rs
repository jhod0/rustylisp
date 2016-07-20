#![feature(io)]
extern crate rustylisp;
use rustylisp::parser::Parser;
use rustylisp::core::Environment;
use rustylisp::evaluator::{self, RuntimeError};

use std::io::{self, Read};
use std::env;
use std::fmt::Debug;

fn eval<I, E>(parser: Parser<I, E>) 
        where I: Iterator<Item=Result<char, E>>, E: Debug {
    let env = evaluator::default_environment().to_env_ref();

    for res in parser {
        match res {
            Ok(obj) => { 
                let new_ref = env.clone();
                match evaluator::eval(obj, new_ref) {
                    Ok(ret) => println!("{}", ret),
                    Err(err) => err.dump_traceback(),
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
            },
        }
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();

    // Read from stdin
    if args.len() < 2 {
        let parser = Parser::new(io::stdin().chars(), String::from("<stdin>"));

        eval(parser);
    } else {
        for file in args.iter().skip(1) {
            println!("Parsing file {}", file);
            let parser = Parser::from_file(file.clone()).expect("file should be valid");

            eval(parser);
        }
    }
}
