use std::{
    borrow::BorrowMut,
    cell::OnceCell,
    io::{Read, Seek, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    process::ExitCode,
    rc::Rc,
    sync::{Arc, RwLock},
};

use branch::alt;
use bytes::complete::{tag, take_while1};
use character::complete::{alphanumeric1, space0};
use combinator::{cut, eof};
use multi::{many1, many_till};
use nom::*;
use sequence::{pair, preceded, terminated};
use serde_json::Value;

use crate::{
    prism::{LiftPrism, Prism, PrismExt},
    Compose, IdentityLens, Lens, Lift, Traversal, TraversalExt,
};

use super::{arr_idx, object_key};

// object keys: alphanumeric (+ _-) string preceded by '.', if u want full unicode fuuuuck off lol (or provide a PR pls)
// array access [usize]
// casting to a type: as k, as y, as z - space preceding 'as'

#[derive(Debug, Clone)]
pub enum Cast {
    Bool,
    Object,
    Array,
    Number,
    String,
}

#[derive(Debug, Clone)]
pub enum Token {
    Key(String),
    Idx(usize),
    // Cast(Cast), // TODO: this will be impl'd last I think - or - just used for type-based narrowing in AST, not for actual casting
}

impl Token {
    pub fn into_selector(self) -> Selector {
        match self {
            Token::Key(k) => Selector(Box::new(JSel(object_key(k)))),
            Token::Idx(x) => Selector(Box::new(JSel(arr_idx(x)))),
            // Token::Cast(_cast) => todo!(), // issue: provides narrowing to subtype - need to lift back up to Value
        }
    }
}

// b/c Prism itself can't be 'dyn'
trait JSelector {
    // must be a dyn Fn so that this can be vtable'd for Selector
    fn over(&self, a: Value, f: &dyn Fn(Value) -> Value) -> Value;

    fn t_to_opt(&self, a: Value) -> Option<Value>;
}

struct JSel<P>(P);

impl<P: Prism<A = Value, B = Value>> JSelector for JSel<P> {
    fn over(&self, a: Value, f: &dyn Fn(Value) -> Value) -> Value {
        <P as TraversalExt>::over(&self.0, a, f)
    }

    fn t_to_opt(&self, a: Value) -> Option<Value> {
        <P as PrismExt>::t_to_opt(&self.0, a)
    }
}

impl<A: JSelector, B: JSelector> JSelector for Compose<A, B> {
    fn over(&self, a: Value, f: &dyn Fn(Value) -> Value) -> Value {
        self.0.over(a, &|a2| self.1.over(a2, f))
    }

    fn t_to_opt(&self, a: Value) -> Option<Value> {
        self.0.t_to_opt(a).and_then(|a| self.1.t_to_opt(a))
    }
}

pub struct Selector(Box<dyn JSelector>);

impl Selector {
    pub fn new<X: JSelector + 'static>(x: X) -> Self {
        Self(Box::new(x))
    }
}

impl JSelector for Selector {
    fn over(&self, a: Value, f: &dyn Fn(Value) -> Value) -> Value {
        self.0.over(a, f)
    }

    fn t_to_opt(&self, a: Value) -> Option<Value> {
        self.0.t_to_opt(a)
    }
}

// NOTE: just promote everything to a traversal? actually wait everything now is only lens - or - prism. no real actual lens here, anyway
// NOTE: just prism and traversal
// pub struct Selector<P: Prism<A=Value, B=Value>>(P);

// impl<P: Prism<A=Value, B=Value>> Selector<P> {

//     pub fn join<O: Prism<A=Value, B=Value>>(self, other: Selector<O>) -> Selector<Compose<P,O>> {
//         Selector(Compose(self.0, other.0))
//     }
// }

#[derive(Debug, Clone)]
pub enum RawExpression {
    Getter(Vec<Token>),
    Setter(Vec<Token>, Setter),
}

impl RawExpression {
    pub fn compile(self) -> Result<Expression, serde_json::Error> {
        let mut sel = Selector::new(JSel(LiftPrism(IdentityLens(PhantomData))));
        match self {
            RawExpression::Getter(tokens) => {
                for token in tokens.into_iter() {
                    sel = Selector::new(Compose(sel, token.into_selector()))
                }

                Ok(Expression::Getter(sel))
            }
            RawExpression::Setter(tokens, value) => {
                for token in tokens.into_iter() {
                    sel = Selector::new(Compose(sel, token.into_selector()))
                }

                Ok(Expression::Setter(
                    sel,
                    serde_json::from_str(value.possible_js_value.as_str())?,
                ))
            }
        }
    }
}

pub enum Expression {
    Getter(Selector),
    Setter(Selector, Value),
}

impl Expression {
    pub fn execute(self, path: PathBuf) -> anyhow::Result<ExitCode> {
        let mut handle = std::fs::File::options().read(true).write(true).open(path)?;
        let mut contents = String::new();
        handle.read_to_string(&mut contents)?;

        let target = serde_json::from_str(contents.as_str())?;

        match self {
            Expression::Getter(selector) => {
                if let Some(res) = selector.t_to_opt(target) {
                    println!("{}", serde_json::to_string_pretty(&res)?);
                    Ok(ExitCode::SUCCESS)
                } else {
                    Ok(ExitCode::FAILURE)
                }
            }
            Expression::Setter(selector, value) => {
                let visited = Arc::new(RwLock::new(false));
                let visited2 = visited.clone();
                let updated = selector.over(target, &move |x| {
                    // janky but w/e
                    let mut visited = visited2.write().unwrap();
                    *visited = true;
                    value.clone()
                });

                if *visited.read().unwrap() {
                    handle.seek(std::io::SeekFrom::Start(0))?;
                    let to_write = serde_json::to_vec(&updated)?;
                    handle.write_all(&to_write[..])?;
                    handle.set_len(to_write.len() as u64)?; // truncate to handle case where to_write is shorter than original
                    Ok(ExitCode::SUCCESS)
                } else {
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
// use serde to parse as value in followup step
// or toml, or etc - cool part is followup PRs can select parser based on file type in input hell yeargh
pub struct Setter {
    possible_js_value: String,
}

pub fn token_parser(input: &str) -> IResult<&str, Token> {
    let (x, c) = alt((
        array_idx_parser.map(Token::Idx),
        obj_key_parser.map(Token::Key),
        // cast_parser.map(Token::Cast)
    ))(input)?;

    Ok((x, c))
}

pub fn setter_parser(input: &str) -> IResult<&str, Setter> {
    let (x, s) = preceded(tag("="), preceded(space0, take_while1(|_| true)))(input)?;

    Ok((
        x,
        Setter {
            possible_js_value: s.to_owned(),
        },
    ))
}

pub fn tokens_parser_2(input: &str) -> IResult<&str, RawExpression> {
    let (x, ts) = alt((
        pair(many1(token_parser), preceded(space0, setter_parser))
            .map(|(t, s)| RawExpression::Setter(t, s)),
        many1(token_parser).map(RawExpression::Getter),
    ))(input)?;

    Ok((x, ts))
}

pub fn tokens_parser(input: &str) -> IResult<&str, RawExpression> {
    let (x, (ts, opt_s)) = many_till(
        token_parser,
        alt((setter_parser.map(Some), eof.map(|_| None))),
    )(input)?;

    let res = match opt_s {
        Some(s) => RawExpression::Setter(ts, s),
        None => RawExpression::Getter(ts),
    };

    Ok((x, res))
}

pub fn obj_key_parser(input: &str) -> IResult<&str, String> {
    let (x, s) = preceded(tag("."), alphanumeric1)(input)?;

    Ok((x, s.to_string()))
}

pub fn array_idx_parser(input: &str) -> IResult<&str, usize> {
    let (x, idx) = terminated(preceded(tag("["), nom::character::complete::u32), tag("]"))(input)?;

    Ok((x, idx as usize)) // if usize is < u32 this panics, but w/e. why is your js array that big? fix your life
}

pub fn cast_type_parser(input: &str) -> IResult<&str, Cast> {
    let (x, c) = alt((
        tag("bool").map(|_| Cast::Bool),
        tag("object").map(|_| Cast::Object),
        tag("array").map(|_| Cast::Array),
        tag("number").map(|_| Cast::Number),
        tag("string").map(|_| Cast::String),
    ))(input)?;

    Ok((x, c))
}

pub fn cast_parser(input: &str) -> IResult<&str, Cast> {
    let (x, c) = preceded(tag("as "), cast_type_parser)(input)?;

    Ok((x, c))
}
