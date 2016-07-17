//! Module for builtin functions
//!
//! Functions are listed in alphabetical order. When possible, they have the same names as their
//! lisp equivalents, but for some (like `+`) this is not possible, and so are named differently.
//!
//! Check BUILTIN_FUNCS to be sure.


mod io;

use ::core::{LispObj, LispObjRef, AsLispObjRef, EnvironmentRef, NativeFunc};
use super::EvalResult;

// TODO add documentation for functions
//
// TODO functions:
//  Numeric comparison: =, <=, >=, <, >
//  Documentation func: doc
//  Absolute equality:  eq?

/// Native functions defined in the default lisp namespace
pub static BUILTIN_FUNCS: &'static [(&'static str, NativeFunc)] = 
    &[("+", add), ("apply", apply), 
      ("bound?", is_bound), ("cons?", is_cons), 
      ("car", car), ("cdr", cdr), ("cons", cons), ("eval", eval), ("list", list),
      ("load-file", io::load_file),
      ("macro-expand", macro_expand)];

/// Builtin values defined in the default lisp namespace.
///
/// Currently, only maps the symbols true and false to themselves.
pub fn builtin_vals() -> Vec<(&'static str, LispObj)> {
    vec![("true", lisp_true!()), ("false", lisp_false!()), ("nil", nil!()), ("*allow-redefine*", lisp_false!())]
}

fn add_two(a: LispObj, b: LispObj) -> EvalResult {
    match (a, b) {
        (LispObj::LInteger(an), LispObj::LInteger(bn))
            => Ok(int!(an + bn)),
        (LispObj::LInteger(an), LispObj::LFloat(bn))
            => Ok(float!((an as f64) + bn)),
        (LispObj::LFloat(an), LispObj::LInteger(bn))
            => Ok(float!(an + (bn as f64))),
        (LispObj::LFloat(an), LispObj::LFloat(bn))
            => Ok(float!(an + bn)),
        (LispObj::LInteger(_), right) 
            => type_error!("expecting number, got {:?}", right),
        (LispObj::LFloat(_), right) 
            => type_error!("expecting number, got {:?}", right),
        (left, _) 
            => type_error!("expecting number, got {:?}", left),
    }
}

pub fn add(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = int!(0);

    for num in args {
        out = try!(add_two(out, (**num).clone()));
    }

    Ok(out)
}

pub fn apply(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => func: Any, arg: Any);
    super::apply(func, arg, env)
}

pub fn is_bound(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(lisp_bool!(env.borrow().lookup(&name).is_some()))
}

pub fn is_cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_cons()))
}

pub fn car(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LCons);
    Ok((*arg.0).clone())
}

pub fn cdr(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LCons);
    Ok((*arg.1).clone())
}

pub fn cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => left: Any, right: Any);
    Ok(cons!(left, right))
}

pub fn eval(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() != 1 {
        syntax_error!("eval expects 1 argument, got {:?}", args.len())
    }

    super::eval(&args[0], env)
}

pub fn list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    Ok(LispObj::to_lisp_list(args.iter()))
}

pub fn macro_expand(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => val: Any);

    match try!(super::macros::try_macro_expand_obj(val.clone(), env)) {
        Some(obj) => Ok((*obj).clone()),
        None => Ok((*val).clone()),
    }
}
