//! The ease-of-use run system
use std::fmt;
use std::io::{self, Read};

use super::core::{LispObj, AsLispObjRef, /* Environment, */ EnvironmentRef, EvalResult};
use super::parser::{self, /* Lexer, */ Parser};
use super::evaluator;

pub struct Evaluator {
    top_level: EnvironmentRef,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            top_level: evaluator::default_environment().to_env_ref()
        }
    }

    pub fn from_existing(env: EnvironmentRef) -> Self {
        Evaluator {
            top_level: env
        }
    }

    fn handle_char(&self, c: char, obj: LispObj) -> Result<LispObj, Option<LispObj>> {
        let handler = match self.top_level.borrow().get_char_handler(c) {
            Some(handler) => handler,
            None => return Err(None),
        };

        evaluator::apply(handler, cons!(obj, nil!()), self.top_level.clone())
                   .map_err(|err| Some(err.into_lisp_obj()))
    }

    pub fn repl(&mut self) {
        let instream = parser::Parser::new(io::stdin().chars(), "<stdin>");

        for obj in instream.with_char_handler(|c, obj| self.handle_char(c, obj)) {
            match obj {
                Ok(obj) => {
                    match evaluator::eval(obj, self.top_level.clone()) {
                        Ok(res)  => println!("{}", res),
                        Err(err) => err.dump_traceback(),
                    }
                },
                Err(err) => {
                    println!("Parse error: {:?}", err);
                },
            }
        }
    }

    pub fn load_from_file(&mut self, path: String) -> EvalResult {
        let file_parser = Parser::from_file(path).unwrap();
        self.eval_all_from_parser(file_parser)
    }

    pub fn eval_all_from_parser<I, E: fmt::Debug, _F>(&mut self, stream: Parser<I, E, _F>) -> EvalResult
            where I: Iterator<Item=Result<char, E>> {
        let mut out = nil!();
        for item in stream.with_char_handler(|c, obj| self.handle_char(c, obj)) {
            out = match item {
                Ok(obj)     => try!(evaluator::eval(obj, self.top_level.clone())),
                Err(err)    => read_error!("{:?}", err),
            };
        }
        Ok(out)
    }
}
