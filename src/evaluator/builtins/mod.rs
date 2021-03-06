//! Module for builtin functions
//!
//! Functions are listed in alphabetical order. When possible, they have the same names as their
//! lisp equivalents, but for some (like `+`) this is not possible, and so are named differently.
//!
//! Check BUILTIN_FUNCS to be sure.
mod io;
mod math;

use std::convert::AsRef;

use ::core::{LispObj, LispObjRef, AsLispObjRef, RuntimeError, EnvironmentRef};
use ::core::obj::{NativeFuncSignature, Procedure};
use ::core::obj::vec::{self, PersistentVec};
use super::EvalResult;

// TODO add documentation for functions
//
// TODO functions:
//  Numeric comparison: =, <=, >=, <, >
//  Absolute equality:  eq?

/// Native functions defined in the default lisp namespace
pub static BUILTIN_FUNCS: &'static [(&'static str, NativeFuncSignature, Option<&'static str>)] = &[
    // Arithmetic
    ("+", math::add, Some(math::ADD_DOCSTR)), ("-", math::sub, Some(math::SUB_DOCSTR)), 
    ("*", math::product, Some(math::PRODUCT_DOCSTR)), ("/", math::division, None),

    // Meta
    ("apply", apply, None), ("doc", doc, None), ("eval", eval, None), ("macro-expand", macro_expand, None),

    // Predicates
    ("bound?",  is_bound, None),  ("cons?",   is_cons, None),
    ("error?",  is_error, None),  ("list?",   is_list, None),
    ("nil?",    is_nil, None),    ("symbol?", is_symbol, None),
    ("string?", is_string, None), ("vector?", is_vector, None),

    // Equality
    ("symbol=?", symbol_eq, None), ("string=?", string_eq, None),

    // Accessors
    ("error-source",  get_error_source, None),
    ("error-type",    get_error_type, None),
    ("error-value",   get_error_value, None),
    ("vector-length", get_vector_length, None),
    ("vector-ref",    get_vector_index, None),
    ("string-length", get_string_length, None),
    ("string-ref",    get_string_index, None),

    ("string", string_append_objects, None),

    // Conversion
    ("list->vector",   list_to_vector, None),
    ("vector->list",   vector_to_list, None),
    ("string->list",   string_to_list, None),
    ("string->symbol", string_to_symbol, None),
    ("symbol->char",   symbol_to_char, None),
    ("symbol->string", symbol_to_string, None),

    // Manipulation & creation
    ("car", car, None), ("cdr", cdr, None), ("cons", cons, None),
    ("make-vector",     make_vector, None),
    ("generate-vector", generate_vector, None),
    ("vector-assoc",    vector_assoc, None),
    ("vector-append",   vector_append, None),
    ("vector-map",      vector_map, None),

    // Error
    ("make-error",  make_error, None),
    ("throw-error", throw_error, None),

    // I/O
    ("change-directory",  io::lisp_set_current_dir, None),
    ("current-directory", io::lisp_get_current_dir, None),
    ("dump-traceback",    dump_traceback, None),
    ("load-file",         io::load_file_handler, None),
    ("pop-directory",     io::lisp_pop_directory, None),
    ("push-directory",    io::lisp_push_directory, None),
    ("print",             io::print, None),
    ("println",           io::println, None),
    ("read",              io::read_handler, None),
];


/// Builtin values defined in the default lisp namespace.
///
/// Currently, only maps the symbols true and false to themselves.
pub fn builtin_vals() -> Vec<(&'static str, LispObj)> {
    vec![("true", lisp_true!()), ("false", lisp_false!()), ("nil", nil!()), ("*allow-redefine*", lisp_false!()),
         (io::DIRECTORY_STACK_NAME, lisp_list![])]
}

pub fn apply(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => func: Any, arg: Any);
    super::apply(func, arg, env)
}

pub fn car(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() != 1 {
        arity_error!("wrong number of arguments to car: {}", LispObj::to_lisp_list(args.iter()));
    } else {
        let arg = &args[0];
        if arg.is_cons() {
            Ok(arg.car().unwrap())
        } else if arg.is_lazy_cons() {
            Ok(arg.lazy_car().unwrap())
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
            Ok(arg.cdr().unwrap())
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
    Ok(cons!(left, right).to_obj_ref())
}

pub fn doc(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => obj: Any);

    match *obj {
        LispObj::LNativeFunc(_, Some(ref docstr), _) => {
            Ok(string!(docstr.as_ref().clone()).to_obj_ref())
        },
        LispObj::LProcedure(box Procedure { documentation: Some(ref docstr), ..}) => {
            Ok(string!(docstr.clone()).to_obj_ref())
        },
        _ => Ok(lisp_false!().to_obj_ref())
    }
}

pub fn dump_traceback(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    for arg in args {
        check_type!(arg, LError).dump_traceback()
    }

    Ok(lisp_true!().to_obj_ref())
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

    Ok(LispObj::LVector(try!(vec)).to_obj_ref())
}

pub fn get_error_source(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => err: LError);
    match &err.source {
        &Some(ref val)  => Ok(val.clone()),
        &None           => Ok(nil!().to_obj_ref()),
    }
}

pub fn get_error_type(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => err: LError);
    Ok(symbol!(err.errname).to_obj_ref())
}

pub fn get_error_value(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => err: LError);
    match &err.value {
        &Some(ref val)  => Ok(val.clone()),
        &None           => Ok(nil!().to_obj_ref()),
    }
}

pub fn get_string_length(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => s: LString);
    Ok(int!(s.len()).to_obj_ref())
}

pub fn get_string_index(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => s: LString, ind: LInteger);
    if ind >= s.len() as i64 {
        argument_error!("string-ref: index {} out of bound of string {:?}", ind, s)
    } else if ind < 0 {
        argument_error!("string-ref: cannot get char at negative index {}", ind)
    } else {
        let i = ind as usize;
        Ok(char!(s.chars().nth(i)
                  .expect("rustylisp::evaluator::builtins::get_string_index: index, len mismatch"))
           .to_obj_ref())
    }
}

pub fn get_vector_length(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => vec: LVector);
    Ok(int!(vec.len()).to_obj_ref())
}

pub fn get_vector_index(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => vec: LVector, ind: LInteger);
    Ok(vec.lookup(ind as usize).map_or(lisp_false!(), |val| (**val).clone()).to_obj_ref())
}

pub fn is_bound(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(lisp_bool!(env.borrow().lookup(&name).is_some()).to_obj_ref())
}

pub fn is_cons(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_cons() || arg.is_lazy_cons()).to_obj_ref())
}

pub fn is_error(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_err()).to_obj_ref())
}

pub fn is_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_list()).to_obj_ref())
}

pub fn is_nil(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_nil()).to_obj_ref())
}

pub fn is_string(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_string()).to_obj_ref())
}

pub fn is_symbol(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_symbol()).to_obj_ref())
}

pub fn is_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(lisp_bool!(arg.is_vector()).to_obj_ref())
}

pub fn list_to_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => list: Any);
    if let Some(n) = list.list_length() {
        let mut list_items = list.list_iter().map(|res| res.unwrap());
        let vec = PersistentVec::from_iter_mut(&mut list_items, n);
        Ok(LispObj::LVector(vec).to_obj_ref())
    } else {
        argument_error!("expected proper list, not {}", list)
    }
}

pub fn macro_expand(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => val: Any);

    match try!(super::macros::try_macro_expand_obj(val.clone(), env)) {
        Some(obj) => Ok(obj),
        None => Ok(val),
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
    Ok(LispObj::make_error(err).to_obj_ref())
}

pub fn make_vector(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => size: LInteger, val: Any);
    let adjsize = if size < 0 {
        argument_error!("cannot make vector of negative size {}", size)
    } else {
        size as usize
    };
    Ok(LispObj::make_vector((0..adjsize).map(|_| val.clone())).to_obj_ref())
}

pub fn string_append_objects(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = String::new();

    for obj in args {
        out.push_str(&format!("{}", obj));
    }

    Ok(string!(out).to_obj_ref())
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
                    return Ok(lisp_false!().to_obj_ref())
                }
            },
            (_, None) => return Ok(lisp_false!().to_obj_ref()),
        };
    }

    Ok(lisp_bool!(out).to_obj_ref())
}

pub fn string_to_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    let chars = string.chars().map(|c| LispObj::LChar(c));
    Ok(LispObj::to_lisp_list(chars).to_obj_ref())
}

pub fn string_to_symbol(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => string: LString);
    Ok(symbol!((*string).clone()).to_obj_ref())
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
                    return Ok(lisp_false!().to_obj_ref())
                }
            },
            (_, None) => return Ok(lisp_false!().to_obj_ref()),
        };
    }

    Ok(lisp_bool!(out).to_obj_ref())
}

pub fn symbol_to_char(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => sym: LSymbol);
    match &sym as &str {
        "space"   => return Ok(LispObj::LChar(' ').to_obj_ref()),
        "tab"     => return Ok(LispObj::LChar('\t').to_obj_ref()),
        "newline" => return Ok(LispObj::LChar('\n').to_obj_ref()),
        _ => {},
    };
    let mut iter = sym.chars();
    let ch = iter.next().unwrap();
    match iter.next() {
        Some(_) => argument_error!("symbol '{} has length >1", sym),
        None => Ok(LispObj::LChar(ch).to_obj_ref()),
    }
}

pub fn symbol_to_string(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol);
    Ok(string!(name).to_obj_ref())
}

pub fn throw_error(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let err = try!(raw_make_error(args, env));
    Err(err)
}

pub fn vector_assoc(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LVector, index: LInteger, item: Any);
    match arg.insert(index as usize, item.clone()) {
        Some(new) => Ok(LispObj::LVector(new).to_obj_ref()),
        None      => {
            runtime_error!("bounds-error", "vector-assoc: index {} is out of bounds of vector {}",
                           index, item)
        }
    }
}

pub fn vector_append(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let vecs: EvalResult<Vec<&PersistentVec<LispObjRef>>> = args.iter().map(|v| {
        if v.is_vector() {
            Ok(v.unwrap_vec())
        } else {
            type_error!("expected vector, not {}", v)
        }
    }).collect();

    Ok(LispObj::LVector(try!(vecs).into_iter()
                                  .flat_map(|v| v.iter())
                                  .map(|obj| obj.clone())
                                  .collect())
       .to_obj_ref())
}

pub fn vector_map(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() < 2 {
        arity_error!("vector-map: not enough arguments {}", LispObj::to_lisp_list(args.iter()))
    }

    let func = args[0].clone();
    let vecs = try!(args[1..].iter().map(|v| {
        if v.is_vector() {
            Ok(v.unwrap_vec().iter())
        } else {
            type_error!("expected vector, not {}", v)
        }
    }).collect::<Result<Vec<_>,_>>());

    struct VecIter<'a>(Vec<vec::Iter<'a,LispObjRef>>);
    impl<'a> Iterator for VecIter<'a> {
        type Item = LispObjRef;
        fn next(&mut self) -> Option<Self::Item> {
            let next_ones: Result<LispObj,_> = self.0.iter_mut().map(|v| {
                match v.next() {
                    Some(x) => Ok(x.clone()),
                    None    => Err(()),
                }
            }).collect();
            match next_ones {
                Ok(v)   => Some(v.to_obj_ref()),
                Err(()) => None
            }
        }
    }

    let iter = VecIter(vecs);
    let new_vec: EvalResult<PersistentVec<LispObjRef>> = iter.map(|args| {
        super::apply(func.clone(), args, env.clone())
               .map(|v| v.to_obj_ref())
    }).collect();
    new_vec.map(|v| LispObj::LVector(v).to_obj_ref())
}

pub fn vector_to_list(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: LVector);
    Ok(LispObj::to_lisp_list(arg.iter()).to_obj_ref())
}
