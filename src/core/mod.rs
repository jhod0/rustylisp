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
/// Creates nil, the empty list
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
/// Quotes a lisp value.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustylisp;
/// # fn main() {
/// let objects = [symbol!("a"), symbol!("b"),
///                lisp_list![int!(1), int!(2), int!(3)],
///                lisp_list![symbol!("a"), symbol!("b"), symbol!("c")]];
///
/// for obj in objects.into_iter() {
///     assert_eq!(quote!(obj), lisp_list![symbol!("quote"), obj]);
/// }
/// # }
/// ```
macro_rules! quote {
    ($val:expr) => ( lisp_list![symbol!("quote"), $val] )
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
/// let raw = LispObj::make_string("Hello, World!");
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
/// Creates a Lisp linked list out of its arguments, in the spirit of vec![]
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustylisp;
/// # use rustylisp::core::AsLispObjRef;
/// # fn main() {
/// let a_list = lisp_list![int!(0), int!(1), int!(2)];
/// let a_vec  = vec![int!(0).to_obj_ref(), int!(1).to_obj_ref(), int!(2).to_obj_ref()];
///
/// assert_eq!(a_list.list_to_vec(), Some(a_vec));
/// # }
/// ```
macro_rules! lisp_list {
    [ $obj:expr ] => {
        cons!($obj, nil!())
    };
    [ $obj:expr, $( $other:expr ),+ ] => {
        cons!($obj, lisp_list!($( $other ),+))
    }
}

/// Creates the canonical true value, the symbol 'true
#[macro_export]
macro_rules! lisp_true {
    () => ( symbol!("true") )
}

/// Creates the canonical false value, the symbol 'false
#[macro_export]
macro_rules! lisp_false {
    () => ( symbol!("false") )
}

/// Converts a rust bool into a lisp boolean.
///
/// A true value is converted to `lisp_true!()`, a false value 
/// becomes `lisp_false!()`.
#[macro_export]
macro_rules! lisp_bool {
    ( $val:expr ) => {
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

pub mod procedure;

pub mod error;
pub use self::error::{RuntimeError, EvalResult};
