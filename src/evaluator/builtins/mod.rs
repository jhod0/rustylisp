//! Module for builtin functions
//!
//! Functions are listed in alphabetical order. When possible, they have the same names as their
//! lisp equivalents, but for some (like `+`) this is not possible, and so are named differently.
//!
//! Check BUILTIN_FUNCS to be sure.
mod io;

use std::convert::AsRef;

use ::core::{LispObj, LispObjRef, AsLispObjRef, RuntimeError, EnvironmentRef};
use ::core::obj::{NativeFuncSignature, Procedure};
use ::core::obj::vec::PersistentVec;
use super::EvalResult;

// TODO add documentation for functions
//
// TODO functions:
//  Numeric comparison: =, <=, >=, <, >
//  Absolute equality:  eq?

/// Native functions defined in the default lisp namespace
pub static BUILTIN_FUNCS: &'static [(&'static str, NativeFuncSignature, Option<&'static str>)] = &[
    // Arithmetic
    ("+", add, Some(ADD_DOCSTR)), ("-", sub, Some(SUB_DOCSTR)), 
    ("*", product, Some(PRODUCT_DOCSTR)), ("/", division, None),

    // Meta
    ("apply", apply, None), ("doc", doc, None), ("eval", eval, None), ("macro-expand", macro_expand, None),

    // Predicates
    ("bound?", is_bound, None),   ("cons?", is_cons, None),
    ("error?", is_error, None),   ("list?", is_list, None),
    ("nil?", is_nil, None),       ("symbol?", is_symbol, None),
    ("string?", is_string, None), ("vector?", is_vector, None),

    // Equality
    ("symbol=?", symbol_eq, None), ("string=?", string_eq, None),

    // Accessors
    ("error-type", get_error_type, None),
    ("error-value", get_error_value, None),
    ("vector-length", get_vector_length, None),
    ("vector-ref", get_vector_index, None),

    ("string", string_append_objects, None),

    // Conversion
    ("list->vector", list_to_vector, None),
    ("vector->list", vector_to_list, None),
    ("string->list", string_to_list, None),
    ("string->symbol", string_to_symbol, None),
    ("symbol->char", symbol_to_char, None),
    ("symbol->string", symbol_to_string, None),

    // Manipulation
    ("car", car, None), ("cdr", cdr, None), ("cons", cons, None),
    ("make-vector", make_vector, None),
    ("vector-assoc", vector_assoc, None),
    ("generate-vector", generate_vector, None),

    // Error
    ("make-error", make_error, None),
    ("throw-error", throw_error, None),

    // I/O
    ("dump-traceback", dump_traceback, None),
    ("load-file", io::load_file_handler, None),
    ("print", io::print, None),
    ("println", io::println, None),
    ("read", io::read_handler, None),
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

const ADD_DOCSTR: &'static str = "Performs addition.

Throws a 'type-error if any arguments are not numbers.

Examples:

(+ 1 2 3)
=> 6";
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

pub fn car(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() != 1 {
        arity_error!("wrong number of arguments to car: {}", LispObj::to_lisp_list(args.iter()));
    } else {
        let arg = &args[0];
        if arg.is_cons() {
            Ok((*arg.car().unwrap()).clone())
        } else if arg.is_lazy_cons() {
            Ok((*arg.lazy_car().unwrap()).clone())
        } else {
            type_error!("car: expected cons, got {}", arg)
        }
    }
}

pub fn cdr(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() != 1 {
        arity_error!("wrong number of arguments to car: {}", LispObj::to_lisp_list(args.iter()));
    } else {
        let arg = &args[0];
        if arg.is_cons() {
            Ok((*arg.cdr().unwrap()).clone())
        } else if arg.is_lazy_cons() {
            let procd = arg.lazy_cdr().unwrap();
            super::lambda::lambda_apply(procd, nil!().to_obj_ref())
        } else {
            type_error!("cdr: expected cons, got {}", arg)
        }
    }
}

pub fn cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => left: Any, right: Any);
    Ok(cons!(left, right))
}

pub fn doc(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => obj: Any);

    match (*obj).clone() {
        LispObj::LNativeFunc(_, Some(ref docstr), _) => {
            Ok(string!(docstr.as_ref().clone()))
        },
        LispObj::LProcedure(Procedure { documentation: Some(ref docstr), ..}) => {
            Ok(string!(docstr.clone()))
        },
        _ => Ok(lisp_false!())
    }
}

pub fn dump_traceback(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    for arg in args {
        check_type!(arg, LError).dump_traceback()
    }

    Ok(lisp_true!())
}

pub fn eval(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() != 1 {
        syntax_error!("eval expects 1 argument, got {}", args.len())
    }

    super::eval(&args[0], env)
}

pub fn generate_vector(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => len: LInteger, fun: Any);
    if !(fun.is_proc() || fun.is_native()) {
        type_error!("generate-vector: expected procedure, not {}", fun)
    }

    let vec: EvalResult<PersistentVec<_>> = (0..len).map(|i| {
        let res = try!(super::apply(fun.clone(), lisp_list![int!(i)], env.clone()));
        Ok(res.to_obj_ref())
    }).collect();

    Ok(LispObj::LVector(try!(vec)))
}

pub fn get_error_type(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => err: LError);
    Ok(symbol!(err.errname))
}

pub fn get_error_value(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => err: LError);
    match &err.value {
        &Some(ref val)  => Ok((**val).clone()),
        &None           => Ok(nil!()),
    }
}

pub fn get_vector_length(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => vec: LVector);
    Ok(int!(vec.len()))
}

pub fn get_vector_index(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => vec: LVector, ind: LInteger);
    Ok(vec.lookup(ind as usize).map_or(lisp_false!(), |val| (**val).clone()))
}

pub fn is_bound(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(lisp_bool!(env.borrow().lookup(&name).is_some()))
}

pub fn is_cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_cons() || arg.is_lazy_cons()))
}

pub fn is_error(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_err()))
}

pub fn is_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_list()))
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

pub fn is_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_vector()))
}

pub fn list_to_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => list: Any);
    if list.is_list() {
        let list_items = list.list_iter().map(|res| res.unwrap());
        Ok(LispObj::make_vector(list_items))
    } else {
        argument_error!("expected proper list, not {}", list)
    }
}

pub fn macro_expand(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => val: Any);

    match try!(super::macros::try_macro_expand_obj(val.clone(), env)) {
        Some(obj) => Ok((*obj).clone()),
        None => Ok((*val).clone()),
    }
}

pub fn raw_make_error(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult<RuntimeError> {
    if args.len() == 0 {
        arity_error!("make-error: no arguments")
    } else if args.len() > 2 {
        arity_error!("make-error: too many arguments")
    }

    let err = args[0].clone();

    let output_err = if err.is_symbol() {
        RuntimeError::error(err.symbol_ref().unwrap())
    } else if err.is_err() {
        err.unwrap_err().clone()
    } else {
        type_error!("make-error: not an error, {}", LispObj::to_lisp_list(args.iter()))
    };

    if args.len() == 2 {
        if args[1].is_err() {
            Ok(output_err.with_cause(args[1].unwrap_err().clone()))
        } else {
            Ok(output_err.with_value(&args[1]))
        }
    } else {
        Ok(output_err)
    }
}

pub fn make_error(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let err = try!(raw_make_error(args, env));
    Ok(LispObj::make_error(err))
}

pub fn make_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => size: LInteger, val: Any);
    let adjsize = if size < 0 {
        argument_error!("cannot make vector of negative size {}", size)
    } else {
        size as usize
    };
    Ok(LispObj::LVector(PersistentVec::repeating(adjsize, val)))
}

const PRODUCT_DOCSTR: &'static str = "Performs multiplication.

Throws a type-error if an argument is not a number.

Examples:

(*)
;; => 1

(* 1 2 3 4)
;; => 12";
pub fn product(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = int!(1);

    for num in args {
        out = try!(mult_two(out, (**num).clone()));
    }

    Ok(out)
}

pub fn string_append_objects(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = String::new();

    for obj in args {
        out.push_str(&format!("{}", obj));
    }

    Ok(string!(out))
}

pub fn string_eq(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out    = true;
    let mut string = None;

    for arg in args {
        out = match (string, arg.string_ref()) {
            (None, Some(obj)) => {
                string = Some(obj);
                true
            },
            (Some(last), Some(obj)) => {
                if last == obj {
                    string = Some(obj);
                    true
                } else {
                    return Ok(lisp_false!())
                }
            },
            (_, None) => return Ok(lisp_false!()),
        };
    }

    Ok(lisp_bool!(out))
}

pub fn string_to_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    let chars = string.chars().map(|c| LispObj::LChar(c));
    Ok(LispObj::to_lisp_list(chars))
}

pub fn string_to_symbol(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    Ok(symbol!((*string).clone()))
}

pub fn symbol_eq(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out  = true;
    let mut symb = None;

    for arg in args {
        out = match (symb, arg.symbol_ref()) {
            (None, Some(obj)) => {
                symb = Some(obj);
                true
            },
            (Some(last), Some(obj)) => {
                if last == obj {
                    symb = Some(obj);
                    true
                } else {
                    return Ok(lisp_false!())
                }
            },
            (_, None) => return Ok(lisp_false!()),
        };
    }

    Ok(lisp_bool!(out))
}

const SUB_DOCSTR: &'static str = "Performs subtraction.

Throws a type-error if an argument is not a number.

(- a b c d e ...)
is equivalent to:
(- a (+ b c d e ...))

(- a)
is equivalent to:
(- 0 a)

Examples:

(- 5 3)
=> 2

(- 10 1 2 3 4)
=> 0";
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

pub fn throw_error(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let err = try!(raw_make_error(args, env));
    Err(err)
}

pub fn vector_assoc(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LVector, index: LInteger, item: Any);
    match arg.insert(index as usize, item.clone()) {
        Some(new) => Ok(LispObj::LVector(new)),
        None      => {
            runtime_error!("bounds-error", "vector-assoc: index {} is out of bounds of vector {}",
                           index, item)
        }
    }
}

pub fn vector_to_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LVector);
    Ok(LispObj::to_lisp_list(arg.iter()))
}
