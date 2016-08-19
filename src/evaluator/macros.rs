use ::core::{EnvironmentRef, LispObjRef, AsLispObjRef, EvalResult};
use ::core::obj::NativeFuncSignature;

pub fn get_handler(name: &str, env: EnvironmentRef) -> Option<LispObjRef> {
    env.borrow().lookup_macro(name)
}

pub fn try_macro_expand_obj(obj: LispObjRef, env: EnvironmentRef) -> EvalResult<Option<LispObjRef>> {
    if let Some((hd, tl)) = obj.cons_split() {
        if let Some(ref s) = hd.symbol_ref() {
            try_macro_expand(s, tl, env)
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub fn try_macro_expand(macro_name: &str, args: LispObjRef, env: EnvironmentRef) -> EvalResult<Option<LispObjRef>> {
    if let Some(handler) = get_handler(macro_name, env.clone()) {
        let macro_expander = handler.unwrap_proc();
        match super::lambda::lambda_apply(macro_expander, args) {
            Ok(val)  => Ok(Some(val.to_obj_ref())),
            Err(err) => macro_error!(cause err; "error in expansion of macro {}", macro_name)
        }
    } else {
        Ok(None)
    }
}


/***************** Special Character Handlers ****************/

pub static SPECIAL_CHAR_DEFAULTS: &'static [(char, NativeFuncSignature)] = 
    &[('\'', quote_handler), ('\\', backslash_handler), 
      ('`', quasiquote_handler), (',', unquote_handler)];

fn backslash_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    super::builtins::symbol_to_char(args, env)
}

fn quasiquote_handler(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(cons!(symbol!("quasiquote"), cons!(arg, nil!())).to_obj_ref())
}

fn quote_handler(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(quote!(arg).to_obj_ref())
}

fn unquote_handler(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);
    Ok(cons!(symbol!("unquote"), cons!(arg, nil!())).to_obj_ref())
}
