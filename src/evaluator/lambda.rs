//! Utilities for working with lisp procedures, both for creation and execution.

use std::rc::Rc;

use ::core::EvalResult;
use ::core::procedure::{ArityObj, Procedure};
use ::core::{LispObjRef, AsLispObjRef,
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

    let fname = func.id;
    loop {
        let (new_env, new_lte) = match last_to_eval.cons_split() {
            Some((hd, tl)) => {

                let env_borrow = env.clone();
                if let Some(sname) = hd.symbol_ref() {

                    // Lookup the symbol and try to get its procedure id
                    let lookup_attempt = env_borrow.borrow().lookup(sname);
                    match lookup_attempt.and_then(|v| v.procedure_id()) {
                        Some(id) => {

                            if id == fname {
                                // Tail call, perform tco!
                                let args = try!(super::map_eval(tl, env.clone()));
                                /* Reuse environment if possible */
                                match Rc::try_unwrap(env) {
                                    Ok(new_env) => {
                                        try!(lambda_apply_until_last_from(func, 
                                                                          args.to_obj_ref(), 
                                                                          new_env.into_inner()))
                                    },
                                    Err(_) => try!(lambda_apply_until_last(func, args.to_obj_ref()))
                                }
                            } 

                            else {
                                // Not a tail-call, normal eval
                                return super::eval(last_to_eval, env)
                            }
                        },

                        // Not a function, do normal eval
                        None => return super::eval(last_to_eval, env)
                    }
                } 

                else {
                    // Non-tail call, vanilla eval
                    return super::eval(last_to_eval, env)
                }
            },

            // Non-call
            None => return super::eval(last_to_eval, env)
        };

        env = new_env; 
        last_to_eval = new_lte;
    }
}

/// Same style as tco functions, check module `rustylisp::evaluator::tco`
pub fn lambda_apply_until_last(func: &Procedure, arg: LispObjRef) -> EvalResult<(EnvironmentRef, LispObjRef)> {
    let (env, body) = try!(start_procedure(func, arg));
    super::tco::special_form_tco_until_last("begin", body, env.to_env_ref())
}

fn lambda_apply_until_last_from(func: &Procedure, arg: LispObjRef, env: Environment) -> EvalResult<(EnvironmentRef, LispObjRef)> {
    let (env, body) = try!(start_procedure_from(func, arg, env));
    super::tco::special_form_tco_until_last("begin", body, env.to_env_ref())
}

pub fn start_procedure(procd: &Procedure, args: LispObjRef) -> EvalResult<(Environment, &[LispObjRef])> {
    let new_env = Environment::from_parent(procd.env.clone());
    start_procedure_from(procd, args, new_env)
}

fn start_procedure_from(procd: &Procedure, args: LispObjRef, mut reuse_env: Environment) -> EvalResult<(Environment, &[LispObjRef])> {
    assert!(procd.body.len() > 0, "Procedure needs at least 1 body");

    for i in 0..(procd.body.len()-1) {
        match parse_args_into(&procd.body[i].0, args.clone(), &mut reuse_env) {
            Ok(()) => {},
            Err(_) => continue,
        }

        return Ok((reuse_env, &procd.body[i].1))
    }

    let last = procd.body.len() - 1;
    try!(parse_args_into(&procd.body[last].0, args, &mut reuse_env));
    Ok((reuse_env, &procd.body[last].1))
}

#[allow(dead_code)]
/// Attempts to parse to a list based on the an arity object, creating a new environment.
pub fn parse_args(arity: &ArityObj, args: LispObjRef, env: EnvironmentRef) -> EvalResult<Environment> {
    let mut new_env = Environment::from_parent(env);
    try!(parse_args_into(arity, args, &mut new_env));
    Ok(new_env)
}

/// Attempts to parse an argument list into an existing environment, based on an arity object.
/// Clears the input environment before loading new names.
pub fn parse_args_into<'a>(arity: &ArityObj, mut args: LispObjRef, env: &'a mut Environment) -> EvalResult<()> {
    env.clear_bindings();

    for (ind, name) in arity.argnames.iter().enumerate() {
        args = match args.cons_split() {
            Some((hd, tl)) => {
                assert!(env.let_new(name.clone(), hd).is_none());
                tl
            },
            None => arity_error!("Too few args: expecting {}, got {}", arity.argnames.len(), ind),
        };
    }

    match arity.rest {
        Some(ref rest_name) => {
            assert!(env.let_new(rest_name.clone(), args).is_none());
            Ok(())
        },
        None => {
            if args.is_nil() {
                Ok(())
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
    let doc = if body.len() > 0 {
        body[0].string_ref().map(|s| (*s).clone())
    } else {
        None
    };

    if let Some(docstr) = doc {
        Ok(Procedure::single_arity(parent, arity, Vec::from(body)).with_doc(docstr))
    } else {
        Ok(Procedure::single_arity(parent, arity, Vec::from(body)))
    }
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

    let docstr = args[0].string_ref();

    for clause in args.iter().skip(if docstr.is_none() {0} else {1}) {
        clauses.push(try!(parse_arglist_body(clause.clone(), parent.clone())));
    }

    // And parse the lambda itself
    if clauses.is_empty() {
        syntax_error!("(case-lambda) must contain at least one clause")
    } else {
        let procd = Procedure::multiple_arity(parent, clauses);
        match docstr {
            Some(s) => Ok(procd.with_doc((*s).clone())),
            None => Ok(procd),
        }
    }
}

fn parse_arglist_body(args: LispObjRef, _: EnvironmentRef) -> EvalResult<(ArityObj, Vec<LispObjRef>)> {
    if let Some((hd, tl)) = args.cons_split() {
        let arity = try!(parse_arglist(hd));
        let body = flatten_list!(tl, "poorly-formed function body");
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
            rest = Some(String::from(arglist.symbol_ref().unwrap()));
            break
        } else {
            arglist = match arglist.cons_split() {
                Some((hd, tl)) => {
                    match hd.symbol_ref() {
                        Some(name) => argnames.push(String::from(name)),
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
