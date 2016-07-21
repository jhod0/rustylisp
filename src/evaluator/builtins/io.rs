use std::io::{self, Read};

use ::core::{env, LispObjRef, AsLispObjRef, EnvironmentRef};
use ::parser::Parser;
use ::evaluator::{self, EvalResult};

pub fn load_file_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => file_path: LString);

    let global = env::get_top_level(env);

    let file_parser = match Parser::from_file(file_path) {
        Ok(file) => file,
        Err(errmsg) => io_error!("cannot open file: {:?}", errmsg),
    };

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

pub fn read_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let mut instream = Parser::new(io::stdin().chars(), "<stdin>");
    match instream.next() {
        Some(Ok(obj))   => Ok(obj),
        Some(Err(err))  => read_error!("{:?}", err),
        None            => read_error!("end of input"),
    }
}
