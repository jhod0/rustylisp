pub use super::{LispObj, LispObjRef, AsLispObjRef, 
                Environment, EnvironmentRef, EvalResult, RuntimeError};


static TCO_BUILTINS: &'static [&'static str] = &["begin", "if", "let"];

// TODO account for macro-expansions

/// Full evaluate
pub fn handle_special_form_tco(form_name: &str, initial_args: &[LispObjRef], env_input: EnvironmentRef) -> EvalResult {
    let (env, val) = try!(special_form_tco_until_last(form_name, initial_args, env_input));
    super::eval(val, env)
}


/// Evaluate until last expression, handling tail call optimization on certain special forms.
/// Returns the last expression to be evaluated as a lisp object.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate rustylisp;
/// # fn main() {
/// use rustylisp::evaluator::{default_environment, tco};
/// use rustylisp::core::AsLispObjRef;
///
/// /* We will evaluate:
///  * (if 'true
///  *     (list 3)
///  *     some-name)
///  *
///  * and:
///  * (begin 'true
///  *        (list 3)
///  *        some-name)
///  */
///
/// let args = vec![lisp_true!().to_obj_ref(), 
///                 cons!(symbol!("list"), cons!(int!(3), nil!())).to_obj_ref(),
///                 symbol!("some-name").to_obj_ref() ];
/// let env = default_environment().to_env_ref();
///
/// let (_, if_res) = tco::special_form_tco_until_last("if", &args, env.clone()).unwrap();
/// assert_eq!(if_res, cons!(symbol!("list"), cons!(int!(3), nil!())));
///
/// let (_, begin_res) = tco::special_form_tco_until_last("begin", &args, env).unwrap();
/// assert_eq!(begin_res, symbol!("some-name"));
/// # }
/// ```
///
/// # Panics
/// Panics when form_name is not a special form. Currently only supports `begin`, `if`, and `let`.
///
/// ```rust,should_panic
/// use rustylisp::evaluator::tco;
/// use rustylisp::core::Environment;
///
/// tco::special_form_tco_until_last("bogus", &[], Environment::empty().to_env_ref());
/// ```
pub fn special_form_tco_until_last(form_name: &str, initial_args: &[LispObjRef], env_input: EnvironmentRef) -> EvalResult<(EnvironmentRef, LispObj)> {
    let mut name = form_name;
    let mut args: Vec<LispObjRef> = initial_args.iter().map(|obj| obj.to_obj_ref()).collect();
    let mut env  = env_input;

    loop {
        let last = match name {
            "begin" => try!(begin_until_last(&args[..], env.clone())),
            "if"    => try!(if_until_last(&args[..], env.clone())),
            "let"   => {
                let (new_env, res) = try!(let_until_last(&args[..], env));
                env = new_env;
                res
            },
            _       => panic!("bogus special form: {}", name)
        };

        /* handle last */
        let (new_name, new_args) = match last.cons_split() {
            Some((hd, tl)) => { 
                if let Some(s) = hd.symbol_ref() {
                    if let Ok(ind) = TCO_BUILTINS.binary_search(&s.as_str()) {
                        let vec = flatten_list!(tl, "ill-formed-list");
                        (TCO_BUILTINS[ind], vec)
                    } else {
                        /* TODO check if s is macro,
                         * check if we can tco its result */
                        return Ok((env, last))
                    }
                } else {
                    return Ok((env, last))
                }
            },
            None => return Ok((env, last)),
        };

        name = new_name; 
        args = new_args;
    }
}


/// The following functions all do partial evaluation:
///     they evaluate most of their bodies, and return the last
///     object to be evaluated.
pub fn begin_until_last(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let len = args.len();

    if len == 0 {
        Ok(nil!())
    } else {
        for i in 0..(len-1) {
            try!(super::eval(&args[i], env.clone()));
        }
        Ok((*args[len-1]).clone())
    }
}

pub fn if_until_last(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    if args.len() != 3 {
        syntax_error!("wrong number of arguments to if: {}", LispObj::to_lisp_list(args.iter()))
    }

    let cond     = args[0].clone();
    let trueval  = args[1].clone();
    let falseval = args[2].clone();

    let truth = try!(super::eval(cond, env));

    if truth.falsey() {
        Ok((*falseval).clone())
    } else {
        Ok((*trueval).clone())
    }
}

/// Let has a different type signature because it can generate a new bindings frame,
/// which would be destroyed if it wasn't returned
pub fn let_until_last(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult<(EnvironmentRef, LispObj)> {
    let new_env = Environment::from_parent(env.clone()).to_env_ref();

    if args.len() < 1 {
        syntax_error!("let must have bindings");
    }

    let bindings = flatten_list!(args[0].clone(), "malformed bindings list");

    /* TODO named let */
    for binding in bindings.into_iter() {
        let unwrapped = flatten_list!(binding, "malformed binding");

        unpack_args!(unwrapped => name: Any, value: Any);
        if !name.is_symbol() {
            syntax_error!("malformed binding: expected symbol, got {}", *name);
        }

        let evaluated = match try!(super::eval(value, new_env.clone())) {
            LispObj::LProcedure(func) => LispObj::LProcedure(func.with_name((*name).clone().unwrap_symbol())),
            other => other,
        };

        /* Associate evaluated with name */
        new_env.borrow_mut().let_new((*name).clone().unwrap_symbol(), evaluated.to_obj_ref());
    }

    let last = try!(begin_until_last(&args[1..], new_env.clone()));

    Ok((new_env, last))
}
