#![feature(rustc_private, quote, box_patterns)]
#![allow(unused_imports, unused_variables, dead_code)]

extern crate lalrpop_util;
extern crate syn;
extern crate syntax;
extern crate proc_macro;

use lalrpop_util::lalrpop_mod;

pub mod token;
pub mod ast;
lalrpop_mod!(pub grammar);
pub mod elaborate;
pub mod codegen;
