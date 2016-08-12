use std::io::{self, Read, Write};
use std::convert::AsRef;

use ::core::{env, LispObj, LispObjRef, AsLispObjRef, EnvironmentRef};
use ::parser::Parser;
use ::evaluator::{self, EvalResult};

pub fn load_file_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => file_path: LString);

    let global = env::get_top_level(env.clone());

    let char_handlers = |c: char, obj: LispObj| {
        let handler = match env.borrow().get_char_handler(c) {
            Some(handler) => handler,
            None => return Err(None)
        };

        evaluator::apply(handler, cons!(obj, nil!()), env.clone())
            .map_err(|err| Some(err.into_lisp_obj()))
    };

    let file_parser = match Parser::from_file((*file_path).clone()) {
        Ok(file) => file,
        Err(errmsg) => io_error!("cannot open file: {:?}", errmsg),
    }.with_char_handler(char_handlers);

    let mut out = nil!();
    for parsed_obj in file_parser {
        let obj = match parsed_obj {
            Ok(obj) => obj,
            Err(e) => io_error!("error parsing file: {:?}", e)
        };
        out = try!(evaluator::eval(obj, global.clone()))
    }

    Ok(out)
}

pub fn print(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    for arg in args.iter() {
        match arg.as_ref() {
            &LispObj::LString(ref s) => print!("{}", s),
            other => print!("{}", other),
        }
    }

    match io::stdout().flush() {
        Ok(_) => {}
        Err(err) => io_error!("{:?}", err)
    }

    Ok(lisp_true!())
}

pub fn println(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let _ = try!(print(args, env));
    println!("");

    match io::stdout().flush() {
        Ok(_) => {}
        Err(err) => io_error!("{:?}", err)
    }

    Ok(lisp_true!())
}

/// TODO handle special chars
pub fn read_handler(_: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let instream = Parser::new(io::stdin().chars(), "<stdin>");
    let top_level = env::get_top_level(env);

    let char_handler = |c, obj| {
        match top_level.borrow().get_char_handler(c) {
            Some(handler) => {
                evaluator::apply(handler, cons!(obj, nil!()), top_level.clone())
                           .map_err(|err| Some(err.into_lisp_obj()))
            },
            None => Err(None),
        }
    };

    match instream.with_char_handler(char_handler).next() {
        Some(Ok(obj))   => Ok(obj),
        Some(Err(err))  => read_error!("{:?}", err),
        None            => read_error!("end of input"),
    }
}

//pub fn 
