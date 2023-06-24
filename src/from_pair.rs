use crate::errors::error;
use crate::parser::{
    parse, parse_one, BinaryExpr, BinaryExprTerm, Break, Call, Declaration, Function, Import, Loop,
    MathOperator, Module, Node, Term, Throw, TryCatch, Type, Variable,
};
use crate::utils::is_unique;
use crate::Rule;
use pest::error::Error;
use pest::iterators::Pair;

pub trait Parse {
    fn parse_from(pair: Pair<'_, Rule>) -> Self;
}

impl Parse for Declaration {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_str().to_string();
        let r#type = inner.next().map(|x| x.into_inner()).map(|mut x| Type {
            ident: x.next().unwrap().as_str().to_string(),
            is_array: x.next().is_some(),
        });

        Self { ident, r#type }
    }
}

impl Parse for Function {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let modifiers: Vec<String> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|modifier| modifier.as_str().trim_end().to_string())
            .collect();

        let declaration = Declaration::parse_from(inner.next().unwrap());

        let raw_args = inner.next().unwrap();
        let start_pos = raw_args.as_span().start_pos();
        let args: Vec<Declaration> = raw_args
            .into_inner()
            .map(|x| Declaration::parse_from(x))
            .collect();

        // Check for duplicate argument idents
        let has_duplicates = !is_unique(args.iter().map(|x| &x.ident));
        if has_duplicates {
            error(Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "Duplicate arguments".to_owned(),
                },
                start_pos,
            ))
        }
        let body = parse(inner.next().unwrap().into_inner());
        Function {
            modifiers,
            declaration,
            args,
            body,
        }
    }
}

impl Parse for Term {
    fn parse_from(pair: Pair<'_, Rule>) -> Term {
        let start_pos = pair.as_span().start_pos();
        match pair.as_rule() {
            Rule::String => Term::String(enquote::unquote(pair.as_str()).unwrap().to_string()),
            Rule::Number => Term::Number(pair.as_str().parse().unwrap()),
            _ => error(Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Unimplemented Term \"{:?}\"", pair.as_rule()).to_owned(),
                },
                start_pos,
            )),
        }
    }
}

impl Parse for Module {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let module_name = inner.next().unwrap().as_str().to_string();
        return Module { ident: module_name };
    }
}

impl Parse for Call {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_str().to_string();
        let fn_args = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|x| Term::parse_from(x.into_inner().next().unwrap()))
            .collect();
        Call {
            ident,
            args: fn_args,
        }
    }
}

impl Parse for Break {
    fn parse_from(_pair: Pair<'_, Rule>) -> Self {
        Break
    }
}

impl Parse for Throw {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_str().to_string();
        Throw { ident }
    }
}
impl Parse for Import {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let ident = inner.next();
        let path = Term::parse_from(ident.unwrap());
        Import { path }
    }
}

impl Parse for Loop {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        Loop {
            body: parse(inner.next().unwrap().into_inner()),
        }
    }
}

impl Parse for TryCatch {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let mut next_tree = || {
            parse(
                inner
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner(),
            )
        };

        let r#try = next_tree();
        let r#catch = next_tree();

        TryCatch { r#try, r#catch }
    }
}

impl Parse for Variable {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        let start_pos = pair.as_span().start_pos();
        let mut inner = pair.into_inner();
        let modifiers: Vec<String> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|modifier| modifier.as_str().trim_end().to_string())
            .collect();
        let declaration = Declaration::parse_from(inner.next().unwrap());
        let value = parse_one(inner.next().unwrap()).unwrap();
        let value = match value {
            Node::Expr(x) => x,
            _ => error(Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "Value is not an expression".to_owned(),
                },
                start_pos,
            )),
        };
        Variable {
            modifiers,
            declaration,
            value,
        }
    }
}

impl Parse for BinaryExpr {
    fn parse_from(pair: Pair<'_, Rule>) -> Self {
        BinaryExpr {
            terms: pair
                .into_inner()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|x| BinaryExprTerm {
                    operand: Term::parse_from((x[0]).clone()),
                    operator: x.get(1).and_then(|x| {
                        match x.clone().into_inner().next().unwrap().as_rule() {
                            Rule::Subtract => Some(MathOperator::Subtract),
                            Rule::Multiply => Some(MathOperator::Multiply),
                            Rule::Divide => Some(MathOperator::Divide),
                            Rule::XOR => Some(MathOperator::XOR),
                            _ => panic!("Unknown operator"),
                        }
                    }),
                })
                .collect::<Vec<_>>(),
        }
    }
}

// TODO: MORE STATEMENTS + EXPRS, AND MAKE MODIFIERS AN ENUM
