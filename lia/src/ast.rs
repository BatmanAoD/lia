use crate::token::LiaToken;
use std::collections::{HashMap, HashSet};
use syn::{Token as RsToken, Ident};
use proc_macro::{TokenTree};

#[derive(Debug, Clone)]
pub enum LiaExpr {
    BinOp(RsToken, Box<LiaExpr>, Box<LiaExpr>),
    Integer(i32),
    String(String),
    Bool(bool),
    Var(Ident),
    RsVar(Vec<Ident>),
    Call(Box<LiaExpr>, Vec<LiaExpr>),
    Closure(Vec<Ident>, Vec<LiaStmt>),
    Object(Vec<(LiaExpr, LiaExpr)>),
    Index(Box<LiaExpr>, Box<LiaExpr>),
    Array(Vec<LiaExpr>),
    Quote(Vec<TokenTree>),
}

#[derive(Debug, Clone)]
pub enum LiaStmt {
    Declare(Ident),
    Assign(LiaExpr, LiaExpr),
    Return(LiaExpr),
    Expr(LiaExpr),
    If(LiaExpr, Vec<LiaStmt>, Option<Vec<LiaStmt>>),
    While(LiaExpr, Vec<LiaStmt>),
    ForObj(Ident, LiaExpr, Vec<LiaStmt>),
}

#[derive(Debug, Clone)]
pub struct LiaFn {
    pub name: Ident,
    pub args: Vec<Ident>,
    pub body: Vec<LiaStmt>,
}

pub fn prefix_ident(id: &Ident, prefix: &str) -> Ident {
    format!("{}{}", prefix, id.name.as_str()).as_str().into()
}

fn get_mapping(mapping: &mut HashMap<Ident, Ident>, id: &Ident) -> Ident {
    if !mapping.contains_key(id) {
        mapping.insert(id.clone(), prefix_ident(id, "_copy"));
    };
    mapping.get(id).expect("Free mapping was invalid").clone()
}


impl LiaExpr {
    pub fn remap_free_vars(
        &mut self,
        bound: &mut HashSet<Ident>,
        mapping: &mut HashMap<Ident, Ident>)
    {
        use self::LiaExpr::*;
        match self {
            &mut Var(ref mut id) => {
                if !bound.contains(id) {
                    *id = get_mapping(mapping, id);
                }
            },
            &mut Closure(ref args, ref mut stmts) => {
                for id in args {
                    bound.insert(id.clone());
                }

                for mut s in stmts.iter_mut() {
                    s.remap_free_vars_aux(bound, mapping);
                }
            },
            &mut BinOp(_, ref mut left, ref mut right) => {
                left.remap_free_vars(bound, mapping);
                right.remap_free_vars(bound, mapping);
            },
            &mut Call(ref mut fun, ref mut args) => {
                fun.remap_free_vars(bound, mapping);
                for arg in args.iter_mut() {
                    arg.remap_free_vars(bound, mapping);
                }
            },
            _ => ()
        }
    }
}

impl LiaStmt {
    pub fn remap_free_vars(&mut self) -> HashMap<Ident, Ident> {
        let mut bound = HashSet::new();
        let mut mapping = HashMap::new();
        self.remap_free_vars_aux(&mut bound, &mut mapping);
        mapping
    }

    pub fn remap_free_vars_aux(
        &mut self,
        bound: &mut HashSet<Ident>,
        mapping: &mut HashMap<Ident, Ident>)
    {
        use self::LiaStmt::*;
        match self {
            &mut Declare(id) => {
                bound.insert(id);
            },
            &mut Assign(ref mut lhs, ref mut rhs) => {
                lhs.remap_free_vars(bound, mapping);
                rhs.remap_free_vars(bound, mapping);
            }
            &mut Return(ref mut expr) => {
                expr.remap_free_vars(bound, mapping);
            },
            &mut Expr(ref mut expr) => {
                expr.remap_free_vars(bound, mapping);
            },
            &mut If(ref mut expr, ref mut if_, ref mut else_) => {
                expr.remap_free_vars(bound, mapping);
                for s in if_.iter_mut() {
                    s.remap_free_vars_aux(bound, mapping);
                }
                if let &mut Some(ref mut else_) = else_ {
                    for s in else_.iter_mut() {
                        s.remap_free_vars_aux(bound, mapping);
                    }
                }
            }
            &mut While(ref mut guard, ref mut body) => {
                guard.remap_free_vars(bound, mapping);
                for s in body.iter_mut() {
                    s.remap_free_vars_aux(bound, mapping);
                }
            }
            &mut ForObj(ref mut id, ref mut expr, ref mut body) => {
                bound.insert(id.clone());
                expr.remap_free_vars(bound, mapping);
                for s in body.iter_mut() {
                    s.remap_free_vars_aux(bound, mapping);
                }

            }
        }
    }
}
