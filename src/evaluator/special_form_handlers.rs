use ::core::{self, LispObj, LispObjRef, AsLispObjRef, EnvironmentRef, NativeFunc, EvalResult};
use super::eval;

/// # Special Form Handlers
///
/// Special forms differ from Native functions in that their arguments are not evaluated
/// prior to being passed to the handler. In this respect they could be described as 
/// "builtin macros", although there is (with a few exceptions) now lower form for them
/// to be translated to.

/* Special forms to handle:
 *
 * TODO get more versatile error handling policy
 *
 * and          - yes
 * begin        - yes
 * case-lambda  - yes
 * catch-error  - yes
 * define       - yes
 * define-macro - partial - need multiple-arity
 * gensym
 * if           - yes
 * let          - partial, need named let
 * lambda       - yes
 * modify!
 * or           - yes
 * quote        - yes
 * quasiquote
 * set!         - yes
 */

pub fn get_handler(s: &str) -> Option<NativeFunc> {
    for &(name, handler) in HANDLERS.iter() {
        if s == name {
            return Some(handler);
        }
    }

    None
}

static HANDLERS: &'static [(&'static str, NativeFunc)] =
      &[("and", and_handler), ("begin", begin_handler), ("case-lambda", case_lambda_handler), ("catch-error", catch_error_handler),  
        ("define", define_handler), ("define-macro", define_macro_handler),
        ("if", if_handler), ("lambda", lambda_handler), ("let", let_handler), ("or", or_handler), ("quote", quote_handler),
        ("set!", set_handler)];

pub fn and_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let mut val = lisp_true!();

    for arg in args.iter() {
        val = try!(super::eval(arg, env.clone()));
        if val.falsey() {
            return Ok(val)
        }
    }

    Ok(val)
}

pub fn begin_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    super::tco::handle_special_form_tco("begin", args, env)
}

pub fn case_lambda_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let func = try!(super::lambda::parse_multiple_arity(args, env));
    Ok(LispObj::LProcedure(func))
}

pub fn catch_error_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    match begin_handler(args, env) {
        Ok(obj)  => Ok(obj),
        Err(err) => Ok(err.into_lisp_obj()),
    }
}

pub fn define_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() < 2 {
        syntax_error!("Not enough arguments to define {:?}", args);
    }

    let (name, value) = if args[0].is_symbol() && args.len() == 2 {
        let name = (*args[0]).clone().unwrap_symbol();
        match try!(eval(args[1].clone(), env.clone())) {
            LispObj::LProcedure(p) => (name.clone(), LispObj::LProcedure(p.with_name(name))),
            val => (name, val)
        }
    } else if let Some((hd, tl)) = args[0].cons_split() {
        if hd.is_symbol() {
            let func_name = hd.symbol_ref().unwrap().clone();
            let func      = try!(super::lambda::parse_lambda_args_body(tl, &args[1..], env.clone()));
            let value     = LispObj::LProcedure(func.with_name(func_name.clone()));

            (func_name, value)
        } else {
            syntax_error!("invalid argumetnts to define: {:?}", &args[0..])
        }
    } else {
        syntax_error!("define must have symbol name to define, not {:?}", args[0])
    };

    let top_level = core::env::get_top_level(env);
    {
        let allow_red = {
            let borrowed = top_level.borrow();
            borrowed.lookup("*allow-redefine*").expect("cannot delete *allow-redefine*")
        };

        let mut borrowed_mut = top_level.borrow_mut();
        match borrowed_mut.let_new(name.clone(), value.to_obj_ref()) {
            Some(_) => {
                if allow_red.falsey() {
                    redefine_error!("symbol {:?} is already bound", name)
                } else {
                    Ok(symbol!(name))
                }
            },
            None => Ok(symbol!(name)),
        }
    }
}

pub fn define_macro_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() < 2 {
        syntax_error!("Not enough arguments to define-macro: {:?}", args)
    }

    if let Some((hd, tl)) = args[0].cons_split() {
        if hd.is_symbol() {
            let macro_name = (*hd).clone().unwrap_symbol();
            let func       = try!(super::lambda::parse_lambda_args_body(tl, &args[1..], env.clone()));
            let value      = LispObj::LProcedure(func.with_name(macro_name.clone()));
            let allow_red  = env.borrow().lookup("*allow-redefine*").expect("cannot delete *allow-redefine*");

            match env.borrow_mut().let_macro(macro_name.clone(), value.to_obj_ref()) {
                Some(_) => {
                    if allow_red.falsey() {
                        redefine_error!("macro {:?} is already bound", macro_name)
                    } else {
                        Ok(symbol!(macro_name))
                    }
                },
                None    => Ok(symbol!(macro_name))
            }
        } else {
            syntax_error!("invalid macro definition: {:?}", args)
        }
    } else {
        syntax_error!("invalid macro definition: {:?}", args)
    }
}

pub fn if_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    super::tco::handle_special_form_tco("if", args, env)
}

pub fn lambda_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let procd = try!(super::lambda::parse_lambda(args, env));
    Ok(LispObj::LProcedure(procd))
}

pub fn let_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    super::tco::handle_special_form_tco("let", args, env)
}

pub fn or_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let mut val = lisp_false!();

    for arg in args.iter() {
        val = try!(super::eval(arg, env.clone()));
        if !val.falsey() {
            return Ok(val)
        }
    }

    Ok(val)
}

pub fn quote_handler(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() == 1 {
        Ok((*args[0]).clone())
    } else {
        syntax_error!("wrong number of arguments to quote")
    }
}

// Set returns the old value of a var
pub fn set_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => name: LSymbol, val: Any);
    let new_value = try!(super::eval(val, env.clone()));

    match env.borrow_mut().swap_values(&name, new_value.to_obj_ref()) {
        Some(old_val) => Ok((*old_val).clone()),
        None => bound_error!("cannot set! unbound symbol {}", name),
    }
}