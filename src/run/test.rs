use ::core::{LispObj, AsLispObjRef, RuntimeError, EvalResult};
use ::evaluator::err_msgs;
use ::parser::Parser;

fn run_test(contents: &str, expected: EvalResult<LispObj>) {
    let mut runner = super::Evaluator::new();
    let parser = Parser::from_string(contents, "<test>");
    let res = runner.eval_all_from_parser(parser);

    match &res {
        &Ok(ref result) => println!("{} => {}", contents, result),
        &Err(ref err) => println!("{} => Err({:?})", contents, err)
    };

    match (expected, res) {
        (Ok(exp), Ok(obj))  => assert_eq!(obj, exp.to_obj_ref()),
        // If error, just check they are the same error type
        (Err(exp), Err(obj)) => assert_eq!(obj.errname, exp.errname),
        (_, _) => assert!(false),
    }
}

macro_rules! tests {
    { $( $( $str:expr ),+ => $expect:expr ),* } => {
        $( $( run_test($str, $expect); )+ )*
    }
}

#[test]
fn test_simple_functions() {
    tests! {
        "(+ 1 2 3 4)"       => Ok(int!(10)),
        "(- 10 1 2 3 4)"    => Ok(int!(0)),
        "(- 3)"             => Ok(int!(-3)),
        "(car '(1 2 3 4))"  => Ok(int!(1)),
        "(cdr '(1 2 3 4))"  => Ok(lisp_list![int!(2), int!(3), int!(4)]),
        "(*)"               => Ok(int!(1)),
        "(* 1 2 3)"         => Ok(int!(6)),
        "(and)"             => Ok(lisp_true!()),
        "(and true 1 2 '(all true values))" 
                            => Ok(lisp_list![symbol!("all"), symbol!("true"), symbol!("values")]),
        "(and 1 () true true)"  => Ok(nil!()),
        "(or)"                  => Ok(lisp_false!()),
        "(or false 1 2 '(all true values))" => Ok(int!(1)),
        "(or '(1 2 3) () true true)"        => Ok(lisp_list![int!(1), int!(2), int!(3)])
    }
}

#[test]
fn test_simple_function_errors() {
    let type_err  = RuntimeError::error(err_msgs::TYPE_ERROR);
    let arity_err = RuntimeError::error(err_msgs::ARITY_ERROR);
    tests! {
        // Math
        "(+ 1 'a)",
        "(- '(1 2 3))",
        "(/ \"hi\" \"there\")",
        "(* 1 2 3 \"a b c\")"   => Err(type_err.clone()),
        "(-)", "(/)"            => Err(arity_err.clone()),

        // Car / cdr / cons
        "(car 'a)", "(cdr 'b)"   => Err(type_err.clone()),
        "(car)", "(car '(1) 2)",
        "(cdr)", "(cdr '(1) 2)"  => Err(arity_err.clone()),

        "(cons)", "(cons 1)",
        "(cons 1 2 3)", "(cons 1 2 3 4)" => Err(arity_err.clone())
    }
}

#[test]
#[ignore] // ignore until named let is implemented
fn test_named_let() {
    tests! {
        "(define (fib n)
            (let f ((a 0) (b 1)
                    (n n))
              (if n b
                  (f b (+ a b) (- n 1)))))
         (list (fib 0) (fib 1) (fib 2) (fib 3) (fib 4))" =>
         Ok(lisp_list![int!(1), int!(1), int!(2), 
                       int!(3), int!(5)])
    }
}
