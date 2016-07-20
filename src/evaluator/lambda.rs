//! Utilities for working with lisp procedures, both for creation and execution.

use ::core::EvalResult;
use ::core::obj::{ArityObj, Procedure};
use ::core::{LispObj, LispObjRef, AsLispObjRef,
             Environment, EnvironmentRef};

/************************** Procedure application ***********************/

// no need to reinvent the wheel, let our special form tco handler
// do this
/// Evaluates the application of a lisp procedurre to its argument list
///
/// If func has a name, does Tail Call Optimization
// TODO clean this!
// TODO reuse environments on tco
pub fn lambda_apply(func: &Procedure, arg: LispObjRef) -> EvalResult {
    let (mut env, mut last_to_eval) = try!(lambda_apply_until_last(func, arg));

    match func.name {
        Some(ref fname) => {
            // let mut lvl = 0;
            loop {
                // println!("application of {}: lvl {}", fname, lvl);
                // println!("\targ = {}", last_to_eval);

                let (new_env, new_lte) = match last_to_eval.cons_split() {
                    Some((hd, tl)) => {
                        if hd.is_symbol() {
                            if fname == hd.symbol_ref().unwrap() {
                                let args = try!(super::map_eval(tl, env.clone()));
                                try!(lambda_apply_until_last(func, args.to_obj_ref()))
                            } else {
                                return super::eval(last_to_eval, env)
                            }
                        } else {
                            return super::eval(last_to_eval, env)
                        }
                    },
                    None => return super::eval(last_to_eval, env)
                };
                env = new_env; 
                last_to_eval = new_lte;

                // lvl += 1;
            }
        },
        None => super::eval(last_to_eval, env)
    }
}

/// Same style as tco functions, check module `rustylisp::evaluator::tco`
pub fn lambda_apply_until_last(func: &Procedure, arg: LispObjRef) -> EvalResult<(EnvironmentRef, LispObj)> {
    let (env, body) = try!(start_procedure(func, arg));
    super::tco::special_form_tco_until_last("begin", body, env.to_env_ref())
}

pub fn start_procedure(procd: &Procedure, args: LispObjRef) -> EvalResult<(Environment, &[LispObjRef])> {
    assert!(procd.body.len() > 0, "Procedure needs at least 1 body");

    for i in 0..(procd.body.len()-1) {
        let env = match parse_args(&procd.body[i].0, args.clone(), procd.env.clone()) {
            Ok(env) => env,
            Err(_) => continue,
        };

        return Ok((env, &procd.body[i].1))
    }

    let last = procd.body.len() - 1;
    let env = try!(parse_args(&procd.body[last].0, args, procd.env.clone()));
    Ok((env, &procd.body[last].1))
}

/// Attempts to parse to a list based on the an arity object, creating a new environment.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rustylisp;
/// # pub fn main() {
/// use rustylisp::core::obj::ArityObj;
/// use rustylisp::core::{LispObj, LispObjRef, AsLispObjRef, Environment};
/// use rustylisp::evaluator::lambda::parse_args;
///
/// // Create an ArityObj for a lambda of the form
/// // (lambda (a b c) ...)
/// let first_arity = ArityObj::new(["a", "b", "c"].iter()
///                                 .map(|s| String::from(*s)).collect(), 
///                             None);
/// let env = Environment::empty().to_env_ref();
///
/// // Apply it to (1 2 3)
/// let args = cons!(int!(1), cons!(int!(2), cons!(int!(3), nil!()))).to_obj_ref();
/// let parsed = parse_args(&first_arity, args, env.clone()).unwrap();
///
/// // So, the environment should be a = 1, b = 2, c = 3
/// for &(name, val) in [("a", 1), ("b", 2), ("c", 3)].iter() {
///     assert_eq!(parsed.lookup(&String::from(name)), Some(int!(val).to_obj_ref()))
/// }
/// # }
/// ```
pub fn parse_args(arity: &ArityObj, mut args: LispObjRef, env: EnvironmentRef) -> EvalResult<Environment> {
    let mut new_env = Environment::from_parent(env);

    for (ind, name) in arity.argnames.iter().enumerate() {
        args = match args.cons_split() {
            Some((hd, tl)) => {
                assert!(new_env.let_new(name.clone(), hd).is_none());
                tl
            },
            None => arity_error!("Too few args: expecting {}, got {}", arity.argnames.len(), ind),
        };
    }

    match arity.rest {
        Some(ref rest_name) => {
            assert!(new_env.let_new(rest_name.clone(), args).is_none());
            Ok(new_env)
        },
        None => {
            if args.is_nil() {
                Ok(new_env)
            } else {
                arity_error!("Extra args: {}", args)
            }
        },
    }
}

/*************************** Procedure Creation ******************************/

/// Parse args and body separately, and construct a single-arity procedure
pub fn parse_lambda_args_body(args: LispObjRef, body: &[LispObjRef], parent: EnvironmentRef) -> EvalResult<Procedure> {
    let arity = try!(parse_arglist(args));

    Ok(Procedure::single_arity(parent, arity, Vec::from(body)))
}

/// Creates a single-arity procedure from its arguments and body, flattened into a single array
pub fn parse_lambda(input: &[LispObjRef], parent: EnvironmentRef) -> EvalResult<Procedure> {
    if input.len() < 1 {
        syntax_error!("lambda must at least have arg-list")
    }

    parse_lambda_args_body(input[0].clone(), &input[1..], parent)
}

/// Parses a series of separate lambda clauses, combining them into a multiple-arity
/// procedure.
pub fn parse_multiple_arity(args: &[LispObjRef], parent: EnvironmentRef) -> EvalResult<Procedure> {
    let mut clauses = vec![];

    for clause in args.iter() {
        clauses.push(try!(parse_arglist_body(clause.clone(), parent.clone())));
    }

    if clauses.is_empty() {
        syntax_error!("(case-lambda) must contain at least one clause")
    } else {
        Ok(Procedure::multiple_arity(parent, clauses))
    }
}

fn parse_arglist_body(args: LispObjRef, env: EnvironmentRef) -> EvalResult<(ArityObj, Vec<LispObjRef>)> {
    if let Some((hd, tl)) = args.cons_split() {
        let arity = try!(parse_arglist(hd));
        let body = flatten_list!(env env; tl, "poorly-formed function body");
        Ok((arity, body))
    } else {
        syntax_error!("invalid lambda expression: {}", cons!(symbol!("lambda"), args))
    }
}

fn parse_arglist(args: LispObjRef) -> EvalResult<ArityObj> {
    let mut argnames = vec![];
    let mut rest     = None;

    let mut arglist = args.clone();
    loop {
        if arglist.is_nil() {
            break
        } else if arglist.is_symbol() {
            rest = Some(arglist.symbol_ref().unwrap().clone());
            break
        } else {
            arglist = match arglist.cons_split() {
                Some((hd, tl)) => {
                    match hd.symbol_ref() {
                        Some(name) => argnames.push(name.clone()),
                        None       => syntax_error!("ill-formed argument list {}", args)
                    };
                    tl
                },
                None => syntax_error!("invalid argument list {}", args)
            }
        }
    }

    Ok(ArityObj::new(argnames, rest))
}
