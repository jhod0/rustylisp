#![feature(io)]
#![feature(range_contains)]
#![feature(inclusive_range_syntax)]

// This order is important, core's macros are used in parser
// and evaluator...
#[macro_use]
pub mod core;
pub mod parser;
// ...and evaluator's macros are used in run
#[macro_use]
pub mod evaluator;
pub mod run;
