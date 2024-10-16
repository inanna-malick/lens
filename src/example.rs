use crate::{Functor, Lens, LensExt as _};

#[derive(Debug, Clone)]
pub struct Molecule {
    pub name: String,
    pub atoms: Vec<Atom>,
}

impl Molecule {
    // todo generic way to construct from functions
    pub fn atoms() -> MoleculeAtoms {
        MoleculeAtoms
    }
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub name: String,
    pub point: Point,
}

impl Atom {
    // todo generic way to construct from functions
    pub fn point() -> AtomPoint {
        AtomPoint
    }

    #[inline(never)]
    pub fn test1(self) -> Atom {
        Atom::point().and(Point::x()).over(self, |x| x + 1)
    }

    #[inline(never)]
    pub fn test2(self) -> Atom {
        Atom {
            point: Point {
                x: self.point.x + 1,
                y: self.point.y,
            },
            name: self.name,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn x() -> PointX {
        PointX
    }
}

pub struct MoleculeAtoms;

impl Lens for MoleculeAtoms {
    type A = Molecule;
    type B = Vec<Atom>;

    fn f<F: Functor>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
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

// pub struct FnLens<A,B,F: Fn(Self::B) -> Functor::F<B> -> Fn(A) -> Functor::F<A>>

pub struct AtomPoint;

impl Lens for AtomPoint {
    type A = Atom;
    type B = Point;

    #[inline(always)]
    fn f<F: Functor>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
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

pub struct PointX;

impl Lens for PointX {
    type A = Point;
    type B = u32;

    #[inline(always)]
    fn f<F: Functor>(
        &self,
        k: impl Fn(Self::B) -> F::F<Self::B>,
    ) -> impl Fn(Self::A) -> F::F<Self::A> {
        move |a| F::fmap(move |new_x| Point { x: new_x, y: a.y }, k(a.x))
    }
}
