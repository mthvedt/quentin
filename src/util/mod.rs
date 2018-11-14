pub mod arena;

use std::hash::{Hash, Hasher};

// TODO: This are not currently used, but we plan to use them for dedup.
pub struct DeepRef<'a, T: 'a + ?Sized> {
    val: &'a T,
}

impl<'a, T: ?Sized> DeepRef<'a, T> {
    pub fn new(val: &'a T) -> Self {
        DeepRef { val: val }
    }

    pub fn borrow<'b>(this: &'b mut DeepRef<'a, T>) -> DeepRef<'b, T> {
        DeepRef { val: this.val }
    }
}

impl<'a, T: ?Sized> Copy for DeepRef<'a, T> {} // needed to remove T: Copy constraint

impl<'a, T: ?Sized> Clone for DeepRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized + PartialEq> PartialEq for DeepRef<'a, T> {
    fn eq(&self, rhs: &DeepRef<T>) -> bool {
        self.val.eq(rhs.val)
    }
}

impl<'a, T: ?Sized + Eq> Eq for DeepRef<'a, T> {}

impl<'a, T: ?Sized + Hash> Hash for DeepRef<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state)
    }
}
