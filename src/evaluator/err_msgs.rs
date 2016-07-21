#[macro_export]
macro_rules! argument_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::ARGUMENT_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! arithmetic_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::ARITHMETIC_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! arity_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::ARITY_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! bound_error {
    ( $( $msg:expr ),*) => {
        runtime_error!( $crate::evaluator::err_msgs::BOUND_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! internal_error {
    ( $( $msg:expr ),*) => {
        runtime_error!( $crate::evaluator::err_msgs::INTERNAL_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! io_error {
    ( $( $msg:expr ),*) => {
        runtime_error!( $crate::evaluator::err_msgs::IO_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! macro_error {
    ( cause $cause:expr; $( $msg:expr ),* ) => {
        runtime_error!( cause $cause; $crate::evaluator::err_msgs::MACRO_ERROR  $(, $msg )* )
    };
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::MACRO_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! read_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::SYNTAX_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! redefine_error {
    ( cause $cause:expr; $( $msg:expr ),* ) => {
        runtime_error!( cause $cause; $crate::evaluator::err_msgs::REDEFINE_ERROR  $(, $msg )* )
    };
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::REDEFINE_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! syntax_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::SYNTAX_ERROR  $(, $msg )* )
    }
}

#[macro_export]
macro_rules! type_error {
    ( $( $msg:expr ),* ) => {
        runtime_error!( $crate::evaluator::err_msgs::TYPE_ERROR  $(, $msg )* )
    }
}

pub static ARGUMENT_ERROR:      &'static str = "argument-error";
pub static ARITHMETIC_ERROR:    &'static str = "arithmetic-error";
pub static ARITY_ERROR:         &'static str = "arity-error";
pub static BOUND_ERROR:         &'static str = "bound-error";
pub static INTERNAL_ERROR:      &'static str = "internal-error";
pub static IO_ERROR:            &'static str = "io-error";
pub static MACRO_ERROR:         &'static str = "macro-expansion-error";
pub static READ_ERROR:          &'static str = "read-error";
pub static REDEFINE_ERROR:      &'static str = "redefine-error";
pub static SYNTAX_ERROR:        &'static str = "syntax-error";
pub static TYPE_ERROR:          &'static str = "type-error";
