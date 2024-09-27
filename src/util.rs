// from https://github.com/rust-lang/rust/issues/20041#issuecomment-2106606655
pub trait TyEq<T> {
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

pub struct Compose<A, B>(pub A, pub B);

// required b/c otherwise the Traversal impl for L: Lens and for (T1, T2) conflict
pub struct Lift<L>(pub L);
