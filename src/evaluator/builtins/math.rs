use ::core::{LispObj, LispObjRef, AsLispObjRef, EvalResult, EnvironmentRef};

enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    fn from_lisp_obj(obj: &LispObj) -> EvalResult<Number> {
        match obj {
            &LispObj::LInteger(n) => Ok(Number::Int(n)),
            &LispObj::LFloat(n)   => Ok(Number::Float(n)),
            val => type_error!("expecting number, got {}", val)
        }
    }

    fn into_lisp_obj(self) -> LispObj {
        match self {
            Number::Int(n)   => LispObj::LInteger(n),
            Number::Float(n) => LispObj::LFloat(n),
        }
    }
}

fn add_two(a: &mut Number, b: &LispObj) -> EvalResult<()> {
    *a = match (&*a, b) {
        (&Number::Int(an), &LispObj::LInteger(bn))
            => Number::Int(an + bn),
        (&Number::Int(an), &LispObj::LFloat(bn))
            => Number::Float((an as f64) + bn),
        (&Number::Float(an), &LispObj::LInteger(bn))
            => Number::Float(an + (bn as f64)),
        (&Number::Float(an), &LispObj::LFloat(bn))
            => Number::Float(an + bn),
        (_, right) 
            => type_error!("expecting number, got {}", right),
    };
    Ok(())
}

fn div_two(a: &mut Number, b: &LispObj) -> EvalResult<()> {
    *a = match (&*a, b) {
        (&Number::Int(an), &LispObj::LInteger(bn))
            => Number::Float((an as f64) / (bn as f64)),
        (&Number::Int(an), &LispObj::LFloat(bn))
            => Number::Float((an as f64) / bn),
        (&Number::Float(an), &LispObj::LInteger(bn))
            => Number::Float(an / (bn as f64)),
        (&Number::Float(an), &LispObj::LFloat(bn))
            => Number::Float(an / bn),
        (_, right) 
            => type_error!("expecting number, got {}", right),
    };

    Ok(())
}

fn mult_two(a: &mut Number, b: &LispObj) -> EvalResult<()> {
    *a = match (&*a, b) {
        (&Number::Int(an), &LispObj::LInteger(bn))
            => Number::Int(an * bn),
        (&Number::Int(an), &LispObj::LFloat(bn))
            => Number::Float((an as f64) * bn),
        (&Number::Float(an), &LispObj::LInteger(bn))
            => Number::Float(an * (bn as f64)),
        (&Number::Float(an), &LispObj::LFloat(bn))
            => Number::Float(an * bn),
        (_, right) 
            => type_error!("expecting number, got {}", right),
    };

    Ok(())
}

fn sub_two(a: &mut Number, b: &LispObj) -> EvalResult<()> {
    *a = match (&*a, b) {
        (&Number::Int(an), &LispObj::LInteger(bn))
            => Number::Int(an - bn),
        (&Number::Int(an), &LispObj::LFloat(bn))
            => Number::Float((an as f64) - bn),
        (&Number::Float(an), &LispObj::LInteger(bn))
            => Number::Float(an - (bn as f64)),
        (&Number::Float(an), &LispObj::LFloat(bn))
            => Number::Float(an - bn),
        (_, right) 
            => type_error!("expecting number, got {}", right),
    };

    Ok(())
}

pub const ADD_DOCSTR: &'static str = "Performs addition.

Throws a 'type-error if any arguments are not numbers.

Examples:

(+ 1 2 3)
=> 6";
pub fn add(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = Number::Int(0);

    for num in args {
        try!(add_two(&mut out, &**num));
    }

    Ok(out.into_lisp_obj().to_obj_ref())
}

pub const SUB_DOCSTR: &'static str = "Performs subtraction.

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
        let mut zero = Number::Int(0);
        try!(sub_two(&mut zero, &*args[0]));
        Ok(zero.into_lisp_obj().to_obj_ref())
    } else {
        let mut out = try!(Number::from_lisp_obj(&*args[0]));

        for num in &args[1..] {
            try!(sub_two(&mut out, &**num))
        }

        Ok(out.into_lisp_obj().to_obj_ref())
    }
}

pub fn division(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() == 0 {
        arity_error!("(/) must have at least 1 argument")
    } else if args.len() == 1 {
        let mut one = Number::Float(1.0);
        try!(div_two(&mut one, &*args[0]));
        Ok(one.into_lisp_obj().to_obj_ref())
    } else {
        let mut out = try!(Number::from_lisp_obj(&*args[0]));

        for num in &args[1..] {
            try!(div_two(&mut out, &**num))
        }

        Ok(out.into_lisp_obj().to_obj_ref())
    }
}

pub const PRODUCT_DOCSTR: &'static str = "Performs multiplication.

Throws a type-error if an argument is not a number.

Examples:

(*)
;; => 1

(* 1 2 3 4)
;; => 12";
pub fn product(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    let mut out = Number::Int(1);

    for num in args {
        try!(mult_two(&mut out, &**num))
    }

    Ok(out.into_lisp_obj().to_obj_ref())
}
