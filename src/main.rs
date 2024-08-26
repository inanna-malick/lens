use std::marker::PhantomData;

fn main() {
    let point = Point { x: 1, y: 1 };

    let atom = Atom {
        name: "helium".to_string(),
        point,
    };

    println!("atom: {:?}", atom);
    println!("atom point: {:?}", getter::<AtomPoint>(atom.clone()));
    println!("atom x: {:?}", getter::<C<AtomPoint, PointX>>(atom.clone()));

    let shift_x = |a| over::<C<AtomPoint, PointX>>(a, |x| x + 1);

    let shifted = shift_x(atom);

    println!("shifted atom: {:?}", shifted);
}

fn getter<L: Lens>(a: L::A) -> L::B {
    L::f::<Const<L::B, Partial>>(|b| Const(b, PhantomData))(a).0
}

fn over<L: Lens>(a: L::A, f: impl Fn(L::B) -> L::B) -> L::A {
    L::f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
}

trait Functor {
    type F<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B>;
}

enum Partial {}

struct Identity<A>(A);

impl Functor for Identity<Partial> {
    type F<A> = Identity<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Identity(f(x.0))
    }
}

impl<X> Functor for Const<X, Partial> {
    type F<A> = Const<X, A>;

    fn fmap<A, B>(_f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Const(x.0, PhantomData)
    }
}

struct Const<A, B>(A, PhantomData<B>);

// type Lens<A, B> = for<F: Functor> B -> F::F<>

// type Lens' a b = forall f . Functor f => (b -> f b) -> (a -> f a)
#[derive(Debug, Clone)]
struct Atom {
    name: String,
    point: Point,
}
#[derive(Debug, Clone)]
struct Point {
    x: u32,
    y: u32,
}

trait Lens {
    type A;
    type B;
    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A>;
}

struct AtomPoint;

impl Lens for AtomPoint {
    type A = Atom;
    type B = Point;

    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        // clone req'd because fmap may call fn multiple times
        // TODO: Try using an FnOnce, see if that breaks anything
        move |a| {
            F::fmap(
                move |new_point| Atom {
                    point: new_point,
                    name: a.name.clone(),
                },
                k(a.point),
            )
        }
    }
}

struct PointX;

impl Lens for PointX {
    type A = Point;
    type B = u32;

    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a| F::fmap(move |new_x| Point { x: new_x, y: a.y }, k(a.x))
    }
}

/// lense compose
struct C<L1, L2>(L1, L2);

impl<L1, L2> Lens for C<L1, L2>
where
    L1: Lens + Sized,
    L2: Lens + Sized,
    L1::B: TyEq<L2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    L1::A: Sized,
    L1::B: Sized,
    L2::A: Sized,
    L2::B: Sized,
{
    type A = L1::A;
    type B = L2::B;

    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = L2::f::<F>(move |b| k(TyEq::rwi(b)));
        L1::f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))

        // let
        // L1::f::<F>(L2::f::<F>(k))
        // todo!()
    }
}

trait TyEq<T> {
    fn rw(self) -> T;
    fn rwi(x: T) -> Self;
}

impl<T> TyEq<T> for T {
    fn rw(self) -> T {
        self
    }
    fn rwi(x: T) -> Self {
        x
    }
}
