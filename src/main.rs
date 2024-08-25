#![feature(non_lifetime_binders)]

use std::marker::PhantomData;

fn main() {
    println!("Hello, world!");
}




trait Functor {
    type F<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B>;
}


struct Identity<A>(A);

struct Const<A,B>(A, PhantomData<B>);

// type Lens<A, B> = for<F: Functor> B -> F::F<>

// type Lens' a b = forall f . Functor f => (b -> f b) -> (a -> f a)

struct Atom{
    name: String,
    point: Point
}

struct Point{x: u32, y: u32}


fn point<F: Functor>(k: impl Fn(Point) -> F::F<Point>, atom: Atom) -> F::F<Atom> {
    // clone req'd because fmap may call fn multiple times
    F::fmap(move |new_point| Atom { point: new_point, name: atom.name.clone() }, k(atom.point))
}

trait Lens<A,B> {
    fn f<F: Functor>(k: impl Fn(B) -> F::F<B>, a: A) -> F::F<A>;
}

struct AtomPoint;

impl Lens<Atom, Point> for AtomPoint {
    fn f<F: Functor>(k: impl Fn(Point) -> F::F<Point>, a: Atom) -> F::F<Atom> {
        // clone req'd because fmap may call fn multiple times
        // TODO: Try using an FnOnce, see if that breaks anything
        F::fmap(move |new_point| Atom { point: new_point, name: a.name.clone() }, k(a.point))
    }
}

/// lense compose
struct LC<A,B,C, L1: Lens<A,B>, L2: Lens<B,C>>(PhantomData<(A,B,C)>, L1, L2);

impl<A,B,C, L1: Lens<A,B>, L2: Lens<B,C>> LC<A,B,C,L1,L2> {
    pub fn new(l1: L1, l2: L2) -> Self {
        Self(PhantomData, l1, l2)
    }
}

impl<L1, L2, Intermediate, A, B> Lens<A,B> for LC<A,Intermediate,B, L1, L2>
  where L1: Lens<A, Intermediate>,
        L2: Lens<Intermediate, B>
{
    fn f<F: Functor>(k: impl Fn(B) -> F::F<B>, a: A) -> F::F<A> {
        let f1 = |k1| |a1| L1::f(k1, a1);
        let f2 = |k2| |a2| L2::f(k2, a2);


        f2(f1(k))(a)
    }
}




// idk if this is viable in rust
// struct Lens<A, B> 
// //   where for<F: Functor> 
// {
//     x: dyn for<F: Functor> Box<dyn Fn(B) -> F::F>
// }








