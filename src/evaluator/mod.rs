//! The lisp evaluator.
/*********************** Macros *************************/
/* (at top so they are available in imports) */

#[macro_export]
macro_rules! unpack_args {
    ( $args:expr => $( $argname:ident: $expect:ident ),+ ) => {
        let mut i = 0;
        $(
            if $args.len() <= i {
                arity_error!("Too few args")
            }
            let $argname = check_type!( $args[i].clone(), $expect );
            i += 1;
        )+

        if $args.len() != i {
            arity_error!("Too many args: expected {}, got {}", i, $args.len())
        }
    }
}

#[macro_export]
macro_rules! check_type {
    ( $val:expr, Any ) => {
        $val
    };
    ( $val:expr, LInt ) => {
        match *($val) {
            $crate::core::LispObj::LInt(n) => n,
            _ => type_error!("expected int, got {}", $val),
        }
    };
    ( $val:expr, LFloat ) => {
        match *($val) {
            $crate::core::LispObj::LFloat(n) => n,
            _ => type_error!("expected int, got {}", $val),
        }
    };
    ( $val:expr, LSymbol ) => {
        match *($val) {
            $crate::core::LispObj::LSymbol(ref name) => name.clone(),
            _ => type_error!("expected symbol, got {}", $val),
        }
    };
    ( $val:expr, LString ) => {
        match *($val) {
            $crate::core::LispObj::LString(ref name) => name.clone(),
            _ => type_error!("expected symbol, got {}", $val),
        }
    };
    ( $val:expr, LCons ) => { 
        {
            let macro_ret: (LispObjRef, LispObjRef) = match $val.cons_split() {
                Some(v) => v,
                None => type_error!("expected cons, got {}", $val),
            };

            macro_ret
        }
    };
}

/// Flattens a lisp list into a Rust vector.
///
/// On failure, throws a `syntax_error` with the given
/// arguments.
///
/// See module `err_msgs` for more on errors.
macro_rules! flatten_list {
    ( env $env:expr; $val:expr, $( $msg:expr ),+ ) => {
        {
            let mut flatten_list_tmp = $val.clone();
            let mut flatten_list_out = vec![];

            while let Some((hd, tl)) = flatten_list_tmp.cons_split() {
                flatten_list_tmp = if hd.is_special_char() {
                    let ch = hd.unwrap_special_char();
                    match $env.borrow().get_char_handler(ch) {
                        Some(handler) => { 
                            if let Some((arg, tail)) = tl.cons_split() {
                                let output = try!($crate::evaluator::apply(handler, cons!(arg, nil!()), $env.clone()));
                                flatten_list_out.push(output.to_obj_ref());
                                tail
                            } else {
                                runtime_error!("reader-error", "no argument to special char: {}", ch)
                            }
                        },
                        None => runtime_error!("reader-error", "no handler for special char: {}", ch)
                    }
                } else {
                    flatten_list_out.push(hd);
                    tl
                }
            }

            if !flatten_list_tmp.is_nil() {
                let flatten_list_macro_msg = format!( $( $msg ),+ );
                syntax_error!("{}: {}", flatten_list_macro_msg, $val)
            } else {
                flatten_list_out
            }
        }
    };
}

/*
#[macro_export]
macro_rules! try_rethrow {
    ( $val:expr, ) => {
        match $val {
            Ok(try_rethrow_macro_obj) => try_rethrow_macro_obj,
        }
    }
}
*/

#[macro_export]
macro_rules! runtime_error {
    ( $name:expr ) => {
        return Err($crate::core::error::RuntimeError::new($name, None, None, None))
    };
    ( $name:expr, $( $msg: expr ),+ ) => {
        { 
            let runtime_err_msg = format!( $( $msg ),+ );
            return Err($crate::core::error::RuntimeError::new($name, Some(string!(runtime_err_msg).to_obj_ref()), None, None))
        }
    };
    ( cause $cause:expr; $name:expr ) => {
        return Err($crate::core::error::RuntimeError::new($name, None, Some($cause), None))
    };
    ( cause $cause:expr; $name:expr $(, $msg: expr)+ ) => {
        { 
            let runtime_err_msg = format!( $( $msg ),+ );
            return Err($crate::core::error::RuntimeError::new($name, Some(string!(runtime_err_msg).to_obj_ref()), Some($cause), None))
        }
    };
}


/********************* Imports ************************/

// There are some useful error reporting macros in err_msgs
#[macro_use]
pub mod err_msgs;

mod builtins;
mod lambda;
mod macros;
mod special_form_handlers;
mod tco;


pub use core::{LispObj, LispObjRef, 
               Environment, EnvironmentRef, AsLispObjRef};
pub use core::{RuntimeError, EvalResult};

/******************** Global evaluator type **************/

pub struct Evaluator {
    global: Environment
}

/******************** Environment Utilities ************************/

pub fn default_environment() -> Environment {
    let bindings = builtins::BUILTIN_FUNCS.iter()
                .map(|&(ref name, ref func)| {
                    (String::from(*name), LispObj::make_native(*name, *func, None).to_obj_ref())
                })
                .chain(builtins::builtin_vals().into_iter()
                    .map(|(name, obj)| (String::from(name), obj.to_obj_ref()))
                );

    let char_handlers = macros::SPECIAL_CHAR_DEFAULTS.iter()
        .map(|&(name, ref func)| {
            (name, LispObj::make_native(format!("char-handler({})", name), *func, None).to_obj_ref())
        });

    Environment::new_with_bindings(bindings).with_special_chars(char_handlers)
}


/******************** The evaluation functions *********************/

fn is_self_evaluating(obj: LispObjRef) -> bool {
    match *obj {
        LispObj::LInteger(_)    => true,
        LispObj::LFloat(_)      => true,
        LispObj::LString(_)     => true,
        LispObj::LNil           => true,
        _ => false,
    }
}

pub fn eval_all<It, Obj>(forms: It, env: EnvironmentRef) -> Result<Vec<LispObj>, RuntimeError> 
        where It: Iterator<Item=Obj>, Obj: AsLispObjRef {
    let mut out = vec![];

    for form in forms {
        let new_env_ref = env.clone();
        out.push(try!(eval(form, new_env_ref)));
    }

    Ok(out)
}

pub fn map_eval(ls: LispObjRef, env: EnvironmentRef) -> EvalResult {
    if ls.is_nil() {
        Ok(nil!())
    } else if let Some((hd, mut tl)) = ls.cons_split() {
        let this = {
            if hd.is_special_char() {
                let ch = hd.unwrap_special_char();
                match env.borrow().get_char_handler(ch) {
                    Some(handler) => { 
                        if let Some((arg, tail)) = tl.cons_split() {
                            let output = try!(apply(handler, cons!(arg, nil!()), env.clone()));
                            tl = tail;
                            try!(eval(output, env.clone()))
                        } else {
                            runtime_error!("reader-error", "no argument to special char: {}", ch)
                        }
                    },
                    None => runtime_error!("reader-error", "no handler for special char: {}", ch)
                }
            } else {
                try!(eval(hd, env.clone()))
            }
        };

        let rest = try!(map_eval(tl, env));
        Ok(cons!(this, rest))
    } else {
        syntax_error!("not a proper list: {}", ls)
    }
}

/// The core of the lisp system: the evaluator
pub fn eval<Obj>(form_input: Obj, env: EnvironmentRef) -> EvalResult
            where Obj: AsLispObjRef {
    let mut form = form_input.to_obj_ref();

    loop {
        // If form is self evaluating, we have nothing to do
        if is_self_evaluating(form.clone()) {
            return Ok((*form).clone());
        }

        // If form is symbol, do lookup
        if let Some(name) = form.symbol_ref() {
            if let Some(val) = env.borrow().lookup(name) {
                return Ok((*val).clone());
            } else {
                bound_error!("symbol '{} is not bound", name)
            }
        }

        if let Some((hd, tl)) =  form.cons_split() {
           // Check if hd is a special form
            if let Some(s) = hd.symbol_ref() {
               // Try special form
               match special_form_handlers::get_handler(s) {
                   Some(handler) => {
                       let tl_vec = flatten_list!(env env; tl, "({}) invalid syntax (ill-formed arg list)", s);
                       return handler(&tl_vec, env)
                   },
                   None => {},
               };

               // Try macros
               match try!(macros::try_macro_expand(s, tl.clone(), env.clone())) {
                   Some(obj) => {
                       form = obj;
                       continue
                   },
                   None => {},
               };
            }

            let func = try!(eval(hd, env.clone()));
            let args = try!(map_eval(tl, env.clone()));
            return apply(func, args, env);
        }

        runtime_error!("eval-error", "unable to evaluate: {}", form)
    }
}

/// Good ole' apply
pub fn apply<Obj1, Obj2>(proc_input: Obj1, arg_input: Obj2, env: EnvironmentRef) -> EvalResult
            where Obj1: AsLispObjRef, Obj2: AsLispObjRef {
    let procedure = proc_input.to_obj_ref();
    let arg = arg_input.to_obj_ref();

    if procedure.is_native() {
        let args = flatten_list!(env env; arg.clone(), "(apply) ill-formed argument list");
        match (*procedure).clone().unwrap_native()(&args, env) {
            Ok(obj) => Ok(obj),
            Err(err) => {
                Err(if err.source.is_some() {
                    RuntimeError::new_from(err, procedure)
                } else {
                    err.with_source(procedure)
                })
            },
        }
    } 

    else if procedure.is_proc() {
        let err = {
            let procd = procedure.unwrap_proc();
            match self::lambda::lambda_apply(procd, arg) {
                Ok(obj) => return Ok(obj),
                Err(err) => err,
            }
        };
        Err(RuntimeError::new_from(err, procedure))
    } 

    else {
        type_error!("expecting procedure, got {}", procedure);
    }
}
