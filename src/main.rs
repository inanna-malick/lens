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
    println!("atom point: {:?}", Atom::point().getter(atom.clone()));
    println!("atom x: {:?}", Atom::point().and(Point::x()).getter(atom.clone()));

    let shifted = Atom::point().and(Point::x()).over(atom, |x| x + 1);

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

    let molecule_x_coords = Molecule::atoms().all(VecElems(PhantomData)).and(Atom::point()).and(Point::x());

    let shifted = molecule_x_coords.over(water, |x| x + 1);
    println!("shifted water: {:?}", shifted);

    let x_coords = molecule_x_coords.to_vec(shifted);
    println!("shifted water x coords: {:?}", x_coords);
}


trait LensExt: Lens {
    fn getter(&self, a: Self::A) -> Self::B {
        self.f::<Const<Self::B, Partial>>(|b| Const(b, PhantomData))(a).0
    }

    fn over(&self, a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
        self.f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    }

    fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Lens<A = Self::A, B = OB> where 
    
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }

    fn all<OA, OB, O: Traversal<A = OA, B = OB>>(self, other: O) -> impl Traversal<A = Self::A, B = OB> where 
    
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(Lift(self), other)
    }
}

impl<L: Lens> LensExt for L {}

trait TraversalExt: Traversal {
    fn to_vec(&self, a: Self::A) -> Vec<Self::B> {
        self.f::<Const<Vec<Self::B>, Partial>>(|b| Const(vec![b], PhantomData))(a).0
    }

    fn over(&self, a: Self::A, f: impl Fn(Self::B) -> Self::B) -> Self::A {
        self.f::<Identity<Partial>>(move |b| Identity(f(b)))(a).0
    }

    fn and<OA, OB, O: Lens<A = OA, B = OB>>(self, other: O) -> impl Traversal<A = Self::A, B = OB> where 
    
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, Lift(other))
    }

    fn all<OA, OB, O: Traversal<A = OA, B = OB>>(self, other: O) -> impl Traversal<A = Self::A, B = OB> where 
    
        Self::B: TyEq<OA>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
    {
        Compose(self, other)
    }
}

impl<T: Traversal> TraversalExt for T {}


#[derive(Debug, Clone)]
struct Molecule {
    name: String,
    atoms: Vec<Atom>,
}

impl Molecule {
    // todo generic way to construct from functions
    pub fn atoms() -> MoleculeAtoms {
        MoleculeAtoms
    }
}

#[derive(Debug, Clone)]
struct Atom {
    name: String,
    point: Point,
}

impl Atom {
    // todo generic way to construct from functions
    pub fn point() -> AtomPoint {
        AtomPoint
    }
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn x() -> PointX {
        PointX
    }
}

trait Lens: Sized {
    type A;
    type B;
    fn f<F: Functor>(&self, k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A>;
}

trait Traversal: Sized {
    type A;
    type B;
    fn f<F: Applicative>(&self, k: impl Fn(Self::B) -> F::F<Self::B>)
        -> impl Fn(Self::A) -> F::F<Self::A>;
}

struct VecElems<X>(PhantomData<X>);

impl<X> Traversal for VecElems<X> {
    type A = Vec<X>;
    type B = X;

    fn f<F: Applicative>(&self, 
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

    fn f<F: Functor>(&self, k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
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

// struct FnLens<A,B,F: Fn(Self::B) -> Functor::F<B> -> Fn(A) -> Functor::F<A>>

struct AtomPoint;

impl Lens for AtomPoint {
    type A = Atom;
    type B = Point;

    fn f<F: Functor>(&self, k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
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

    fn f<F: Functor>(&self, k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a| F::fmap(move |new_x| Point { x: new_x, y: a.y }, k(a.x))
    }
}

impl<L1, L2> Lens for Compose<L1, L2>
where
    L1: Lens,
    L2: Lens,
    L1::B: TyEq<L2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = L1::A;
    type B = L2::B;

    fn f<F: Functor>(&self, k: impl Fn(Self::B) -> F::F<Self::B>) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = self.1.f::<F>(move |b| k(TyEq::rwi(b)));
        self.0.f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    }
}

struct Compose<A, B> (A, B);

impl<T1, T2> Traversal for Compose<T1, T2>
where
    T1: Traversal,
    T2: Traversal,
    T1::B: TyEq<T2::A>, // NEED TO WITNESS THAT THESE TYPES ARE THE SAME SOME-FUCKING-HOW
{
    type A = T1::A;
    type B = T2::B;

    fn f<F: Applicative>(&self, 
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        let k2 = self.1.f::<F>(move |b| k(TyEq::rwi(b)));
        self.0.f::<F>(move |b| F::fmap(TyEq::rwi, k2(TyEq::rw(b))))
    }
}

// required b/c otherwise the Traversal impl for L: Lens and for (T1, T2) conflict
struct Lift<L>(L);

impl<L: Lens> Traversal for Lift<L> {
    type A = L::A;
    type B = L::B;

    fn f<F: Applicative>(
        &self, k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        self.0.f::<F>(k)
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
