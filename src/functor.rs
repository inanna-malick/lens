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

    #[inline(always)]
    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Identity(f(x.0))
    }
}

impl Applicative for Identity<Partial> {
    #[inline(always)]
    fn pure<A>(a: A) -> Self::F<A> {
        Identity(a)
    }

    #[inline(always)]
    fn seq<A, B>(a: Self::F<A>, b: Self::F<B>) -> Self::F<(A, B)> {
        Identity((a.0, b.0))
    }
}

impl<X> Functor for Const<X, Partial> {
    type F<A> = Const<X, A>;

    #[inline(always)]
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
        a.extend(b);
        a
    }
}

impl<X: Monoid> Applicative for Const<X, Partial> {
    fn pure<A>(_a: A) -> Self::F<A> {
        Const(X::zero(), PhantomData)
    }

    fn seq<A, B>(a: Self::F<A>, b: Self::F<B>) -> Self::F<(A, B)> {
        Const(X::concat(a.0, b.0), PhantomData)
    }
}

impl<X> Functor for Vec<X> {
    type F<A> = Vec<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        x.into_iter().map(f).collect()
    }
}

pub struct Const<A, B>(pub A, pub PhantomData<B>);

// man idk at this point I'm just porting haskell types I barely understand
// https://hackage.haskell.org/package/profunctors-5.6.2/docs/src/Data.Profunctor.Unsafe.html#Profunctor
pub trait ProFunctor {
    type P<A, B>;

    fn dimap<A, B, C, D>(
        f1: impl Fn(A) -> B,
        f2: impl Fn(C) -> D,
        x: Self::P<B, C>,
    ) -> Self::P<A, D>;
}

// man idk, it's this: https://hackage.haskell.org/package/profunctors-5.6.2/docs/src/Data.Profunctor.Choice.html#Choice
pub trait Choice: ProFunctor {
    // left'  :: p a b -> p (Either a c) (Either b c)
    // left' =  dimap (either Right Left) (either Right Left) . right'
    // right' :: p a b -> p (Either c a) (Either c b)
    // right' =  dimap (either Right Left) (either Right Left) . left'

    // arbitrary choice: implementers must define left', right' is provided here
    // also Result is used instead of Either, no semantic value for Ok vs Err, just Left/Right here

    fn left<A, B, C>(p: Self::P<A, B>) -> Self::P<Result<A, C>, Result<B, C>>;

    fn right<A, B, C>(p: Self::P<A, B>) -> Self::P<Result<C, A>, Result<C, B>> {
        Self::dimap(
            |a| match a {
                Ok(a) => Err(a),
                Err(c) => Ok(c),
            },
            |b| match b {
                Ok(b) => Err(b),
                Err(c) => Ok(c),
            },
            Self::left(p),
        )
    }
}
