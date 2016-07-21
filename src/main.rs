#![feature(io)]
extern crate rustylisp;
use rustylisp::run;

use std::io::{self, Read};
use std::env;
use std::fmt::Debug;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut env = run::Evaluator::new();

    // Read from stdin
    if args.len() == 1 {
        env.repl()
    } else {
        for file in args.into_iter().skip(1) {
            println!("Parsing file {}", &file);
            match env.load_from_file(file) {
                Ok(obj) => println!("{}", obj),
                Err(err) => err.dump_traceback()
            }
        }
    }
}
