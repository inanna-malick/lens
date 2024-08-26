use std::marker::PhantomData;

pub trait Functor {
    type F<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B>;
}

pub trait Applicative: Functor {
    fn pure<A>(a: A) -> Self::F<A>;
    fn seq<A, B>(a: Self::F<A>, b: Self::F<B>) -> Self::F<(A, B)>;
}

pub enum Partial {}

pub struct Identity<A>(pub A);

impl Functor for Identity<Partial> {
    type F<A> = Identity<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Identity(f(x.0))
    }
}

impl Applicative for Identity<Partial> {
    fn pure<A>(a: A) -> Self::F<A> {
        Identity(a)
    }

    fn seq<A, B>(a: Self::F<A>, b: Self::F<B>) -> Self::F<(A, B)> {
        Identity((a.0, b.0))
    }
}

impl<X> Functor for Const<X, Partial> {
    type F<A> = Const<X, A>;

    fn fmap<A, B>(_f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Const(x.0, PhantomData)
    }
}

pub trait Monoid {
    fn zero() -> Self;
    fn concat(a: Self, b: Self) -> Self;
}

impl<X> Monoid for Vec<X> {
    fn zero() -> Self {
        Vec::new()
    }

    fn concat(mut a: Self, b: Self) -> Self {
        a.extend(b.into_iter());
        a
    }
}

impl<X: Monoid> Applicative for Const<X, Partial> {
    fn pure<A>(_a: A) -> Self::F<A> {
        Const(X::zero(), PhantomData)
    }

    fn seq<A, B>(a: Self::F<A>, b: Self::F<B>) -> Self::F<(A, B)> {
        Const(X::concat(a.0,b.0), PhantomData)
    }
}

impl<X> Functor for Vec<X> {
    type F<A> = Vec<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        x.into_iter().map(f).collect()
    }
}


pub struct Const<A, B>(pub A, pub PhantomData<B>);
