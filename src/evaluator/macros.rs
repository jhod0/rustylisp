use ::core::{Environment, EnvironmentRef, LispObj, LispObjRef, AsLispObjRef, EvalResult};

pub fn get_handler(name: &String, env: EnvironmentRef) -> Option<LispObjRef> {
    env.borrow().lookup_macro(&*name)
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

pub fn try_macro_expand(macro_name: &String, args: LispObjRef, env: EnvironmentRef) -> EvalResult<Option<LispObjRef>> {
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
