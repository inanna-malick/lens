#![feature(non_lifetime_binders)]

use std::marker::PhantomData;

fn main() {
    let point = Point{x: 1, y: 1};

    let atom = Atom{
        name: "helium".to_string(),
        point
    };


    println!("atom: {:?}", atom);
    println!("atom point: {:?}", getter(AtomPoint)(atom.clone()) );
    println!("atom x: {:?}", getter(LC::new(AtomPoint, PointX))(atom.clone()));


    let shiftX = over(LC::new(AtomPoint, PointX), |x| x + 1);

    let shifted = shiftX(atom);


    println!("shifted atom: {:?}", shifted);

}


fn getter<A, B, L: Lens<A,B>>(l: L) -> impl Fn(A) -> B {
    |a| L::f::<Const<B, Partial>>(|b| Const(b, PhantomData))(a).0
}

fn over<A, B, L: Lens<A,B>>(l: L,  f: impl Fn(B) -> B) -> impl Fn(A) -> A {
    move |a| L::f::<Identity<Partial>>(|b| Identity(f(b)))(a).0
}



trait Functor {
    type F<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B>;
}


enum Partial{}

struct Identity<A>(A);

impl Functor for Identity<Partial> {
    type F<A> = Identity<A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Identity(f(x.0))
    }
}

impl<X> Functor for Const<X, Partial> {
    type F<A> = Const<X, A>;

    fn fmap<A, B>(f: impl Fn(A) -> B, x: Self::F<A>) -> Self::F<B> {
        Const(x.0, PhantomData)
    }
}

struct Const<A,B>(A, PhantomData<B>);

// type Lens<A, B> = for<F: Functor> B -> F::F<>

// type Lens' a b = forall f . Functor f => (b -> f b) -> (a -> f a)
#[derive(Debug, Clone)]
struct Atom{
    name: String,
    point: Point
}
#[derive(Debug, Clone)]
struct Point{x: u32, y: u32}


fn point<F: Functor>(k: impl Fn(Point) -> F::F<Point>, atom: Atom) -> F::F<Atom> {
    // clone req'd because fmap may call fn multiple times
    F::fmap(move |new_point| Atom { point: new_point, name: atom.name.clone() }, k(atom.point))
}

trait Lens<A,B> {
    fn f<F: Functor>(k: impl Fn(B) -> F::F<B>) -> impl Fn(A) -> F::F<A>;
}

struct AtomPoint;

impl Lens<Atom, Point> for AtomPoint {
    fn f<F: Functor>(k: impl Fn(Point) -> F::F<Point>) -> impl Fn(Atom) -> F::F<Atom> {
        // clone req'd because fmap may call fn multiple times
        // TODO: Try using an FnOnce, see if that breaks anything
        move |a| F::fmap(move |new_point| Atom { point: new_point, name: a.name.clone() }, k(a.point))
    }
}

struct PointX;

impl Lens<Point, u32> for PointX {
    fn f<F: Functor>(k: impl Fn(u32) -> F::F<u32>) -> impl Fn(Point) -> F::F<Point> {
        move |a| F::fmap(move |new_x| Point { x: new_x, y: a.y }, k(a.x))
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
    fn f<F: Functor>(k: impl Fn(B) -> F::F<B>) -> impl Fn(A) -> F::F<A> {

       L1::f::<F>(L2::f::<F>(k))
    }
}




// idk if this is viable in rust
// struct Lens<A, B> 
// //   where for<F: Functor> 
// {
//     x: dyn for<F: Functor> Box<dyn Fn(B) -> F::F>
// }








