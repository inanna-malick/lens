use std::marker::PhantomData;

mod functor;
use functor::*;

fn main() {
    let point = Point { x: 1, y: 1 };

    let atom = Atom {
        name: "helium".to_string(),
        point,
    };

    println!("atom: {:?}", atom);
    println!("atom point: {:?}", AtomPoint::getter(atom.clone()));
    println!("atom x: {:?}", <(AtomPoint, PointX)>::getter(atom.clone()));

    let shifted = <(AtomPoint, PointX)>::over(atom, |x| x + 1);

    println!("shifted atom: {:?}", shifted);

    let water = Molecule {
        name: "water".to_string(),
        atoms: vec![
            Atom {
                name: "hydrogen".to_string(),
                point: Point { x: 0, y: 0 },
            },
            Atom {
                name: "hydrogen".to_string(),
                point: Point { x: 1, y: 1 },
            },
            Atom {
                name: "oxygen".to_string(),
                point: Point { x: 2, y: 2 },
            },
        ],
    };

    println!("water: {:?}", water);

    let shifted = <(Lift<MoleculeAtoms>, (VecElems<Atom>, (Lift<AtomPoint>, Lift<PointX>)))>::over(water, |x| x + 1);
    println!("shifted water: {:?}", shifted);

    let x_coords = <(Lift<MoleculeAtoms>, (VecElems<Atom>, (Lift<AtomPoint>, Lift<PointX>)))>::to_vec(shifted);
    println!("shifted water x coords: {:?}", x_coords);
}

// NOTE: the type level approach is neat but it preclues building aeson-lens style functionality
//       specifically, how would I construct a lens that looks a string key up from a hashmap at
//       typelevel? u can't. also const type param are u128,u32,char,bool only - no str


trait LensExt: Lens {
    fn getter(a: Self::A) -> Self::B {
        Self::f::<Const<Self::B, Partial>>(|b| Const(b, PhantomData))(a).0
    }

    fn over(a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
        Self::f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    }
}

impl<L: Lens> LensExt for L {}

trait TraversalExt: Traversal {
    fn to_vec(a: Self::A) -> Vec<Self::B> {
        Self::f::<Const<Vec<Self::B>, Partial>>(|b| Const(vec![b], PhantomData))(a).0
    }

    fn over(a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
        Self::f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    }
}

impl<T: Traversal> TraversalExt for T {}


#[derive(Debug, Clone)]
struct Molecule {
    name: String,
    atoms: Vec<Atom>,
}

#[derive(Debug, Clone)]
struct Atom {
    name: String,
    point: Point,
}
#[derive(Debug, Clone, Copy)]
struct Point {
    x: u32,
    y: u32,
}

trait Lens {
    type A;
    type B;
    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A>;
}

trait Traversal {
    type A;
    type B;
    fn f<F: Applicative>(k: impl Fn(Self::B) -> F::F<Self::B>)
        -> impl Fn(Self::A) -> F::F<Self::A>;
}

struct VecElems<X>(PhantomData<X>);

impl<X> Traversal for VecElems<X> {
    type A = Vec<X>;
    type B = X;

    fn f<F: Applicative>(
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a: Vec<X>| {
            a.into_iter().fold(F::pure(Vec::new()), |acc, i: X| {
                F::fmap(
                    |(mut elems, elem): (_, X)| {
                        elems.push(elem);
                        elems
                    },
                    F::seq(acc, k(i)),
                )
            })
        }
    }
}

struct MoleculeAtoms;

impl Lens for MoleculeAtoms {
    type A = Molecule;
    type B = Vec<Atom>;

    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a| {
            F::fmap(
                move |new_atoms| Molecule {
                    atoms: new_atoms,
                    name: a.name.clone(),
                },
                k(a.atoms),
            )
        }
    }
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

impl<L1, L2> Lens for (L1, L2)
where
    L1: Lens,
    L2: Lens,
    L1::B: TyEq<L2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = L1::A;
    type B = L2::B;

    fn f<F: Functor>(k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = L2::f::<F>(move |b| k(TyEq::rwi(b)));
        L1::f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    }
}

impl<T1, T2> Traversal for (T1, T2)
where
    T1: Traversal,
    T2: Traversal,
    T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = T1::A;
    type B = T2::B;

    fn f<F: Applicative>(
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = T2::f::<F>(move |b| k(TyEq::rwi(b)));
        T1::f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    }
}

// required b/c otherwise the Traversal impl for L: Lens and for (T1, T2) conflict
struct Lift<L>(PhantomData<L>);

impl<L: Lens> Traversal for Lift<L> {
    type A = L::A;
    type B = L::B;

    fn f<F: Applicative>(
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        L::f::<F>(k)
    }
}

// from https://github.com/rust-lang/rust/issues/20041#issuecomment-2106606655
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
