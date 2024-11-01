use std::{borrow::Cow, cell::OnceCell, collections::HashMap};

use serde_json::{Map, Number, Value};

pub mod parse;

use crate::{
    prism::{idx, Prism, VecIdx},
    Applicative, Compose, Traversal,
};

// todo: read _and_ mutate js files, with motivation being mutation:
//       via a series of operations js "filename" 'foo.bar = []'

pub struct JObject;

impl Traversal for JObject {
    type A = Value;

    type B = serde_json::Map<String, Value>;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a {
            Value::Object(o) => F::fmap(move |o| Value::Object(o), k(o)),
            _ => F::pure(a),
        }
    }
}

impl Prism for JObject {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

pub struct MapKey<'a> {
    key: Cow<'a, str>, // todo: use cow later
}

impl<'a> Prism for MapKey<'a> {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

// NOTE: only does the thing if object exists at that path. maybe make it create if path not viable? would be neat,
//       for use case of creating objects on the cmd line in a file. ok, but could have code at a _higher level_ do that. yes.
//       great. do that, preserve 'over id x == x' property, allow for reading each path segment one at a time
impl<'a> Traversal for MapKey<'a> {
    type A = serde_json::Map<String, Value>;

    type B = Value;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |m: serde_json::Map<String, Value>| {
            let v: Option<Value> = m.get(self.key.as_ref()).cloned();

            match v {
                Some(v) => F::fmap(
                    move |v2| {
                        let mut m = m.clone(); // TODO: remove somehow

                        m.insert(self.key.clone().into_owned(), v2);

                        m
                    },
                    k(v),
                ),
                None => F::pure(m),
            }
        }
    }
}

pub fn object_key<'x, X: Into<Cow<'x, str>>>(x: X) -> Compose<JObject, MapKey<'x>> {
    Compose(JObject, MapKey { key: x.into() })
}

pub fn arr_idx(x: usize) -> Compose<JArray, VecIdx<Value>> {
    Compose(JArray, idx(x))
}

struct JString;

impl Traversal for JString {
    type A = Value;

    type B = String;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a {
            Value::String(s) => F::fmap(move |s| Value::String(s), k(s)),
            _ => F::pure(a),
        }
    }
}

impl Prism for JString {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

pub struct JArray;

impl Traversal for JArray {
    type A = Value;

    type B = Vec<Value>;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a {
            Value::Array(a) => F::fmap(move |a| Value::Array(a), k(a.to_owned())),
            _ => F::pure(a),
        }
    }
}

impl Prism for JArray {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

struct JNumber;

impl Traversal for JNumber {
    type A = Value;

    type B = Number;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a.as_number() {
            Some(n) => F::fmap(move |n| Value::Number(n), k(n.to_owned())),
            None => F::pure(a),
        }
    }
}

impl Prism for JNumber {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

struct JBool;

impl Traversal for JBool {
    type A = Value;

    type B = bool;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a.as_bool() {
            Some(b) => F::fmap(move |b| Value::Bool(b), k(b)),
            None => F::pure(a),
        }
    }
}

impl Prism for JBool {
    // Assertion: object key is a traversal that will return 0 or 1 only
}

// pub struct ValueObject();

// // todo: actually a prism
// impl Traversal for ValueObject {
//     // todo: lifetimes!
//     type A = Value;
//     type B = Map<String, Value>;
//     fn f<F: Applicative>(
//         &self,
//         k: impl Fn(Self::B) -> F::F<Self::B>,
//     ) -> impl Fn(Self::A) -> F::F<Self::A> {
//         move |a: Value| match a {
//             Value::Object(map) => F::fmap(Value::Object, k(map)),
//             x => F::pure(x),
//         }
//     }
// }
