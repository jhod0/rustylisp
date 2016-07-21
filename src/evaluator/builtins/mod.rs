//! Module for builtin functions
//!
//! Functions are listed in alphabetical order. When possible, they have the same names as their
//! lisp equivalents, but for some (like `+`) this is not possible, and so are named differently.
//!
//! Check BUILTIN_FUNCS to be sure.
mod io;

use ::core::{LispObj, LispObjRef, AsLispObjRef, EnvironmentRef};
use ::core::obj::NativeFuncSignature;
use super::EvalResult;

// TODO add documentation for functions
//
// TODO functions:
//  Numeric comparison: =, <=, >=, <, >
//  Documentation func: doc
//  Absolute equality:  eq?

/// Native functions defined in the default lisp namespace
pub static BUILTIN_FUNCS: &'static [(&'static str, NativeFuncSignature)] = &[
    ("+", add), ("-", sub), ("*", product), ("/", division),

    // Meta
    ("apply", apply), ("eval", eval), ("macro-expand", macro_expand),

    // Predicates
    ("bound?", is_bound), ("cons?", is_cons), ("nil?", is_nil),
    ("symbol?", is_symbol), ("string?", is_string),

    // Conversion
    ("string->list", string_to_list),
    ("string->symbol", string_to_symbol),
    ("symbol->char", symbol_to_char),
    ("symbol->string", symbol_to_string),

    // Manipulation
    ("car", car), ("cdr", cdr), ("cons", cons),

    // I/O
    ("load-file", io::load_file_handler),
    ("read", io::read_handler),
];


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
            => type_error!("expecting number, got {}", right),
        (LispObj::LFloat(_), right) 
            => type_error!("expecting number, got {}", right),
        (left, _) 
            => type_error!("expecting number, got {}", left),
    }
}

fn div_two(a: LispObj, b: LispObj) -> EvalResult {
    match (a, b) {
        (LispObj::LInteger(an), LispObj::LInteger(bn))
            => Ok(float!((an as f64) / (bn as f64))),
        (LispObj::LInteger(an), LispObj::LFloat(bn))
            => Ok(float!((an as f64) / bn)),
        (LispObj::LFloat(an), LispObj::LInteger(bn))
            => Ok(float!(an / (bn as f64))),
        (LispObj::LFloat(an), LispObj::LFloat(bn))
            => Ok(float!(an / bn)),
        (LispObj::LInteger(_), right) 
            => type_error!("expecting number, got {}", right),
        (LispObj::LFloat(_), right) 
            => type_error!("expecting number, got {}", right),
        (left, _) 
            => type_error!("expecting number, got {}", left),
    }
}

fn mult_two(a: LispObj, b: LispObj) -> EvalResult {
    match (a, b) {
        (LispObj::LInteger(an), LispObj::LInteger(bn))
            => Ok(int!(an * bn)),
        (LispObj::LInteger(an), LispObj::LFloat(bn))
            => Ok(float!((an as f64) * bn)),
        (LispObj::LFloat(an), LispObj::LInteger(bn))
            => Ok(float!(an * (bn as f64))),
        (LispObj::LFloat(an), LispObj::LFloat(bn))
            => Ok(float!(an * bn)),
        (LispObj::LInteger(_), right) 
            => type_error!("expecting number, got {}", right),
        (LispObj::LFloat(_), right) 
            => type_error!("expecting number, got {}", right),
        (left, _) 
            => type_error!("expecting number, got {}", left),
    }
}

fn sub_two(a: LispObj, b: LispObj) -> EvalResult {
    match (a, b) {
        (LispObj::LInteger(an), LispObj::LInteger(bn))
            => Ok(int!(an - bn)),
        (LispObj::LInteger(an), LispObj::LFloat(bn))
            => Ok(float!((an as f64) - bn)),
        (LispObj::LFloat(an), LispObj::LInteger(bn))
            => Ok(float!(an - (bn as f64))),
        (LispObj::LFloat(an), LispObj::LFloat(bn))
            => Ok(float!(an - bn)),
        (LispObj::LInteger(_), right) 
            => type_error!("expecting number, got {}", right),
        (LispObj::LFloat(_), right) 
            => type_error!("expecting number, got {}", right),
        (left, _) 
            => type_error!("expecting number, got {}", left),
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

pub fn division(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() == 0 {
        arity_error!("(/) must have at least 1 argument")
    } else if args.len() == 1 {
        div_two(int!(1), (*args[0]).clone())
    } else {
        let mut out = (*args[0]).clone();

        for num in &args[1..] {
            out = try!(div_two(out, (**num).clone()));
        }

        Ok(out)
    }
}


pub fn is_bound(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(lisp_bool!(env.borrow().lookup(&name).is_some()))
}

pub fn is_cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_cons()))
}

pub fn is_nil(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_nil()))
}

pub fn is_string(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_string()))
}

pub fn is_symbol(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_symbol()))
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
        syntax_error!("eval expects 1 argument, got {}", args.len())
    }

    super::eval(&args[0], env)
}

pub fn macro_expand(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => val: Any);

    match try!(super::macros::try_macro_expand_obj(val.clone(), env)) {
        Some(obj) => Ok((*obj).clone()),
        None => Ok((*val).clone()),
    }
}

pub fn product(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = int!(1);

    for num in &args[1..] {
        out = try!(mult_two(out, (**num).clone()));
    }

    Ok(out)
}

pub fn string_to_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    let chars = string.chars().map(|c| LispObj::LChar(c));
    Ok(LispObj::to_lisp_list(chars))
}

pub fn string_to_symbol(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    Ok(symbol!(string))
}

pub fn sub(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() == 0 {
        arity_error!("(-) must have at least one argument")
    } else if args.len() == 1 {
        sub_two(int!(0), (*args[0]).clone())
    } else {
        let mut out = (*args[0]).clone();

        for num in &args[1..] {
            out = try!(sub_two(out, (**num).clone()));
        }

        Ok(out)
    }
}

pub fn symbol_to_char(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => sym: LSymbol);
    match &sym as &str {
        "space"   => return Ok(LispObj::LChar(' ')),
        "tab"     => return Ok(LispObj::LChar('\t')),
        "newline" => return Ok(LispObj::LChar('\n')),
        _ => {},
    };
    let mut iter = sym.chars();
    let ch = iter.next().unwrap();
    match iter.next() {
        Some(_) => argument_error!("symbol '{} has length >1", sym),
        None => Ok(LispObj::LChar(ch)),
    }
}

pub fn symbol_to_string(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(string!(name))
}
