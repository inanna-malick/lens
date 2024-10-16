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

pub trait Prism: Sized {
    type A;
    type B;
    // type Prism a b = forall p f. (Choice p, Applicative f) => p a (f a) -> p b (f b)
    // (simplified from s t a b formulation)
    fn f<F: Applicative, P: Choice>(
        &self,
        p: P::P<Self::A, F::F<Self::A>>,
    ) -> P::P<Self::B, F::F<Self::B>>;
}

// impl<X> Prism for Option<X> {
// type A = Option<X>;
// type B = X;
//
// fn f<F: Applicative, P: Choice>(
// &self,
// p: P::P<Self::A, F::F<Self::A>>,
// ) -> P::P<Self::B, F::F<Self::B>> {
// let p1: P::P<Result<Self::A, _>, Result<F::F<Self::A>, _>> = P::left(p);
// P::dimap(|b| Ok(Some(b)), |rfa| match fa {
//     Ok(fa) => {
//         <F as Functor>::fmap(|a|  , fa)
//     }
//     Err(idk) => {}
// }, p1)
// P::left(p)
// }
// }

pub trait PrismExt: Prism {
    // fn get_if(&self, a: Self::A) -> Option {
    //     // self.f::<Const<Vec<Self::B>, Partial>>(|b| Const(vec![b], PhantomData))(a).0
    //     // self.f::<Const<Leftmost<Self::B>, Partial>, _>(|b| Const(vec![b], PhantomData))(a).0

    //     let p = self.f::<Fn(Partial, _) -> Partial>(|a, b| Const::<Leftmost<Self::B>, Partial>::pure(a));

    // }

    // fn over(&self, a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
    //     self.f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    // }

    // fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
    // where
    //     Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    // {
    //     Compose(self, Lift(other))
    // }

    fn and_if<OA, OB, O: Prism<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }

    // fn all<OA, OB, O: Prism<A = OA, B = OB>>(
    //     self,
    //     other: O,
    // ) -> impl Traversal<A = Self::A, B = OB>
    // where
    //     Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    // {
    //     Compose(self, other)
    // }
}

impl<P: Prism> PrismExt for P {}

impl<T1, T2> Prism for Compose<T1, T2>
where
    T1: Prism,
    T2: Prism,
    T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = T1::A;
    type B = T2::B;

    // fucking hell idk revisit later
    fn f<F: Applicative, P: Choice>(
        &self,
        p: P::P<Self::A, F::F<Self::A>>,
    ) -> P::P<Self::B, F::F<Self::B>> {
        self.0.f::<F, P>(p)
    }
    
    // fn f<F: Applicative, P: Choice>(
    //     &self,
    //     k: impl Fn(Self::B) -> F::F<Self::B>,
    // ) -> impl Fn(Self::A) -> F::F<Self::A> {
    //     let k2 = self.1.f::<F, P>(move |b| k(TyEq::rwi(b)));
    //     self.0.f::<F, P>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    // }
}

// impl<L: Lens> Prism for Lift<L> {
//     type A = L::A;
//     type B = L::B;

//     fn f<F: Applicative>(
//         &self,
//         k: impl Fn(Self::B) -> F::F<Self::B>,
//     ) -> impl Fn(Self::A) -> F::F<Self::A> {
//         self.0.f::<F>(k)
//     }
// }
