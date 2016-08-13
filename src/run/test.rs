use ::core::EvalResult;
use ::parser::Parser;

fn run_test(contents: &str, expected: EvalResult) {
    let mut runner = super::Evaluator::new();
    let parser = Parser::from_string(contents, "<test>");
    let res = runner.eval_all_from_parser(parser);
    assert_eq!(res, expected)
}

macro_rules! tests {
    { $( $str:expr => $expect:expr ),* } => {
        $( run_test($str, $expect); )*
    }
}

#[test]
fn test_simple_functions() {
    tests! {
        "(+ 1 2 3 4)" => Ok(int!(10)),
        "(- 10 1 2 3 4)" => Ok(int!(0)),
        "(- 3)" => Ok(int!(-3)),
        "(car '(1 2 3 4))" => Ok(int!(1)),
        "(cdr '(1 2 3 4))" => Ok(lisp_list![int!(2), int!(3), int!(4)])
    }
}
