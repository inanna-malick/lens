use std::marker::PhantomData;

use crate::{Applicative, Compose, Const, Functor, Identity, Lens, Lift, Partial, Traversal, TyEq};



pub trait Prism: Sized {
    type A;
    type B;
    fn f<F: Applicative>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A>;
}

// pub trait PrismExt: Prism {
//     fn t_to_vec(&self, a: Self::A) -> Vec<Self::B> {
//         self.f::<Const<Vec<Self::B>, Partial>>(|b| Const(vec![b], PhantomData))(a).0
//     }

//     fn over(&self, a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
//         self.f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
//     }

//     fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
//     where
//         Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
//     {
//         Compose(self, Lift(other))
//     }

//     fn and_if<OA, OB, O: Prism<A = OA, B = OB>>(self, other: O) -> impl Prism<A = Self::A, B = OB>
//     where
//         Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
//     {
//         Compose(self, other)
//     }

//     fn all<OA, OB, O: Prism<A = OA, B = OB>>(
//         self,
//         other: O,
//     ) -> impl Traversal<A = Self::A, B = OB>
//     where
//         Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
//     {
//         Compose(self, other)
//     }
// }

// impl<T: Prism> PrismExt for T {}

// impl<T1, T2> Prism for Compose<T1, T2>
// where
//     T1: Prism,
//     T2: Prism,
//     T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
// {
//     type A = T1::A;
//     type B = T2::B;

//     fn f<F: Applicative>(
//         &self,
//         k: impl Fn(Self::B) -> F::F<Self::B>,
//     ) -> impl Fn(Self::A) -> F::F<Self::A> {
//         let k2 = self.1.f::<F>(move |b| k(TyEq::rwi(b)));
//         self.0.f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
//     }
// }

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
