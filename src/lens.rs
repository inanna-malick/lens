use std::marker::PhantomData;

use crate::{Compose, Const, Functor, Identity, Lift, Partial, Traversal, TyEq};

pub trait Lens: Sized {
    type A;
    type B;
    fn f<F: Functor>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A>;
}

impl<L1, L2> Lens for Compose<L1, L2>
where
    L1: Lens,
    L2: Lens,
    L1::B: TyEq<L2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = L1::A;
    type B = L2::B;

    fn f<F: Functor>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = self.1.f::<F>(move |b| k(TyEq::rwi(b)));
        self.0.f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    }
}

pub trait LensExt: Lens {
    fn getter(&self, a: Self::A) -> Self::B {
        self.f::<Const<Self::B, Partial>>(|b| Const(b, PhantomData))(a).0
    }

    fn over(&self, a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
        self.f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    }

    fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Lens<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }

    fn all<OA, OB, O: Traversal<A = OA, B = OB>>(
        self,
        other: O,
    ) -> impl Traversal<A = Self::A, B = OB>
    where
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(Lift(self), other)
    }
}

impl<L: Lens> LensExt for L {}
