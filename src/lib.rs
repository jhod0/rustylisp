#![feature(io)]
#![feature(range_contains)]
#![feature(inclusive_range_syntax)]

// This order is important, core's macros are used in parser
// and evaluator
#[macro_use]
pub mod core;

/// The lexing and parsing system. 
pub mod parser;

pub mod evaluator;
