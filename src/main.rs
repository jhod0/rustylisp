extern crate rustylisp;
use rustylisp::run;

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut env = run::Evaluator::new();

    // Read from stdin
    if args.len() == 1 {
        env.repl()
    } else {
        for file in args.into_iter().skip(1) {
            match env.load_from_file(file) {
                Ok(obj) => println!("{}", obj),
                Err(err) => err.dump_traceback()
            }
        }
    }
}
