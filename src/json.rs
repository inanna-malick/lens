use serde_json::{Map, Value};

use crate::{Applicative, Traversal};

pub struct ObjectKey {
    pub key: String, // todo: use cow later
}

impl Traversal for ObjectKey {
    type A = Map<String, Value>;

    type B = Value;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Map<String, Value>| match a.get(self.key.as_str()).cloned() {
            Some(val) => F::fmap(
                move |val2| {
                    let mut a2 = a.clone();

                    a2.insert(self.key.clone(), val2);

                    a2
                },
                k(val),
            ),
            None => F::pure(a),
        }
    }
}

pub struct ValueObject();

// todo: actually a prism
impl Traversal for ValueObject {
    // todo: lifetimes!
    type A = Value;

    type B = Map<String, Value>;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Value| match a {
            Value::Object(map) => F::fmap(Value::Object, k(map)),
            x => F::pure(x),
        }
    }
}
