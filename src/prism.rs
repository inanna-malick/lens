use std::marker::PhantomData;

use crate::{
    Applicative, Choice, Compose, Const, Functor, Identity, Lens, Lift, Monoid, Partial, Traversal,
    TyEq,
};

// Maybe monoid returning the leftmost non-Nothing value.
// https://hackage.haskell.org/package/base-4.18.1.0/docs/Data-Monoid.html#t:First
struct First<A>(A);

// Leftmost for https://hackage.haskell.org/package/lens-5.3.2/docs/Control-Lens-Fold.html#v:firstOf
enum Leftmost<A> {
    Found(A),
    None,
}

impl<A> Leftmost<A> {
    fn to_opt(self) -> Option<A> {
        match self {
            Leftmost::Found(a) => Some(a),
            Leftmost::None => None,
        }
    }
}

impl<A> Monoid for Leftmost<A> {
    fn zero() -> Self {
        Leftmost::None
    }

    fn concat(a: Self, b: Self) -> Self {
        match (a, b) {
            (Leftmost::Found(a), _) => Leftmost::Found(a),
            (Leftmost::None, Leftmost::Found(b)) => Leftmost::Found(b),
            (Leftmost::None, Leftmost::None) => Leftmost::None,
        }
    }
}

// The weird Profunctor/Choice thing is (1) hard and (2) scary so this is just a special case of traversal
pub trait Prism: Traversal {}

pub trait PrismExt: Prism {
    fn t_to_opt(&self, a: Self::A) -> Option<Self::B> {
        self.f::<Const<Leftmost<Self::B>, Partial>>(|b| Const(Leftmost::Found(b), PhantomData))(a)
            .0
            .to_opt()
    }

    fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, Lift(other))
    }

    fn and_if<OA, OB, O: Prism<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }

    fn all<OA, OB, O: Prism<A = OA, B = OB>>(self, other: O) -> impl Traversal<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }
}

impl<T1, T2> Prism for Compose<T1, Lift<T2>>
where
    T1: Prism,
    T2: Lens,
    T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
}

impl<T1, T2> Prism for Compose<T1, T2>
where
    T1: Prism,
    T2: Prism,
    T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
}

pub struct LiftPrism<P>(pub P);

impl<L: Lens> Traversal for LiftPrism<L> {
    type A = L::A;

    type B = L::B;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        self.0.f::<F>(k)
    }
}

impl<L: Lens> Prism for LiftPrism<L> {}

impl<T: Prism> PrismExt for T {}

pub struct VecIdx<X> {
    idx: usize,
    vec_value_type: PhantomData<X>,
}

pub fn idx<X>(idx: usize) -> VecIdx<X> {
    VecIdx {
        idx,
        vec_value_type: PhantomData,
    }
}

impl<X: Clone> Prism for VecIdx<X> {}

// NOTE: only does the thing if object exists at that path. maybe make it create if path not viable? would be neat,
//       for use case of creating objects on the cmd line in a file. ok, but could have code at a _higher level_ do that. yes.
//       great. do that, preserve 'over id x == x' property, allow for reading each path segment one at a time
impl<X: Clone> Traversal for VecIdx<X> {
    type A = Vec<X>;

    type B = X;

    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |mut vec: Vec<X>| {
            // let vec2 = vec.clone(); // TODO make conditional on item existing
            let item: Option<&X> = vec.get(self.idx);

            match item {
                Some(item) => {
                    let vec2 = vec.clone(); // TODO: remove somehow
                    F::fmap(
                        move |item2| {
                            let mut vec2 = vec2.clone(); // TODO: remove somehow

                            vec2.insert(self.idx, item2);

                            vec2
                        },
                        k(item.clone()),
                    )
                }
                None => F::pure(vec),
            }
        }
    }
}
