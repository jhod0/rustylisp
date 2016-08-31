//! The ease-of-use run system
#[cfg(test)]
mod test;

use std::convert::AsRef;
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

        evaluator::apply(handler, lisp_list!(obj), self.top_level.clone())
                   .map(|obj| (*obj).clone())
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

    // TODO mimic load-file and change directories
    pub fn load_from_file<P: AsRef<::std::path::Path>>(&mut self, path: P) -> EvalResult {
        let file_parser = Parser::from_file(path).unwrap();
        self.eval_all_from_parser(file_parser)
    }

    pub fn eval_all_from_parser<I, E: fmt::Debug, _F>(&mut self, stream: Parser<I, E, _F>) -> EvalResult
            where I: Iterator<Item=Result<char, E>> {
        let mut out = nil!().to_obj_ref();
        let source_name = String::from(stream.source_name());
        for item in stream.with_char_handler(|c, obj| self.handle_char(c, obj)) {
            out = match item {
                Ok(obj)     => try!(evaluator::eval(obj, self.top_level.clone())),
                Err(err)    => {
                    println!("error on input: {}", source_name);
                    read_error!("{:?}", err)
                }
            };
        }
        Ok(out)
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        self.top_level.borrow_mut().clear_bindings()
    }
}
