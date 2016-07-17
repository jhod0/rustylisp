use ::core::{env, LispObj, LispObjRef, AsLispObjRef, EnvironmentRef};
use ::parser::Parser;
use ::evaluator::{self, EvalResult};

pub fn load_file(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
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
