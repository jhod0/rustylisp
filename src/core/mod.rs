//! The core of the Lisp system. Contains definitions of all basic lisp objects
//!
//! Contains the definition of Lisp objects (`LispObj`), as well as the definition of an
//! environment (`Environment`), as well as their respective reference types.

#[macro_export]
/// Creates a symbol from a string
macro_rules! symbol {
    ($name:expr) => ( $crate::core::LispObj::make_symbol($name) )
}

#[macro_export]
/// Creates Nil, the empty list
macro_rules! nil {
    () => ( $crate::core::LispObj::LNil )
}

#[macro_export]
/// Constructs a cons cell of two LispObjects
macro_rules! cons {
    ($car:expr, $cdr:expr) => ( $crate::core::LispObj::cons($car, $cdr) )
}

#[macro_export]
/// Creates a LispObj integer
macro_rules! int {
    ($n:expr) => ( $crate::core::LispObj::LInteger($n as i64) )
}

#[macro_export]
/// Creates a LispObj float
macro_rules! float {
    ($n:expr) => ( $crate::core::LispObj::LFloat($n as f64) )
}

#[macro_export]
macro_rules! quote {
    ($val:expr) => ( cons!(symbol!("quote"), cons!($val, nil!())) )
}

#[macro_export]
/// Creates a LispObj string
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustylisp;
/// # fn main() {
/// use rustylisp::core::LispObj;
///
/// let raw = LispObj::LString("Hello, World!".to_string());
/// // Isn't this much nicer?
/// let with_macro = string!("Hello, World!");
///
/// assert_eq!(with_macro, raw);
/// # }
/// ```
macro_rules! string {
    ($str:expr) => ( $crate::core::LispObj::make_string($str) )
}

#[macro_export]
macro_rules! lisp_list {
    [ $obj:expr ] => {
        cons!($obj, nil!())
    };
    [ $obj:expr, $( $other:expr ),+ ] => {
        cons!($obj, lisp_list!($( $other ),+))
    }
}

#[macro_export]
macro_rules! lisp_true {
    () => ( symbol!("true") )
}

#[macro_export]
macro_rules! lisp_false {
    () => ( symbol!("false") )
}

#[macro_export]
macro_rules! lisp_bool {
    ( $val:expr) => {
        if $val {
            lisp_true!()
        } else {
            lisp_false!()
        }
    }
}

pub mod obj;
pub use self::obj::{LispObj, LispObjRef, AsLispObjRef, NativeFunc};

pub mod env;
pub use self::env::{Environment, EnvironmentRef};

pub mod error;
pub use self::error::{RuntimeError, EvalResult};
