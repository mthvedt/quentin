//! Reference wrappers that implement Hash and Eq.

use std::hash::{Hash, Hasher};

pub struct ShallowRef<'a, T: 'a + ?Sized> {
    val: &'a T,
}

impl <'a, T: ?Sized> ShallowRef<'a, T> {
    pub fn new(val: &'static T) -> Self {
        ShallowRef {
            val: val,
        }
    }

    pub fn borrow<'b>(this: &'b mut ShallowRef<'a, T>) -> ShallowRef<'b, T> {
        ShallowRef {
            val: this.val,
        }
    }
}

impl <'a, T: ?Sized> Copy for ShallowRef<'a, T> {} // needed to remove T: Copy constraint

impl <'a, T: ?Sized> Clone for ShallowRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <'a, T: ?Sized> PartialEq for ShallowRef<'a, T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.val as *const _ == rhs.val as *const _
    }
}

impl <'a, T: ?Sized> Eq for ShallowRef<'a, T> {}

impl <'a, T: ?Sized> Hash for ShallowRef<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.val as *const T).hash(state)
    }
}

pub struct DeepRef<'a, T: 'a + ?Sized> {
    val: &'a T,
}

impl <'a, T: ?Sized> DeepRef<'a, T> {
    pub fn new(val: &'static T) -> Self {
        DeepRef {
            val: val,
        }
    }

    pub fn borrow<'b>(this: &'b mut DeepRef<'a, T>) -> DeepRef<'b, T> {
        DeepRef {
            val: this.val,
        }
    }
}

impl <'a, T: ?Sized> Copy for DeepRef<'a, T> {} // needed to remove T: Copy constraint

impl <'a, T: ?Sized> Clone for DeepRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <'a, T: ?Sized + PartialEq> PartialEq for DeepRef<'a, T> {
    fn eq(&self, rhs: &DeepRef<T>) -> bool {
        self.val.eq(rhs.val)
    }
}

impl <'a, T: ?Sized + Eq> Eq for DeepRef<'a, T> {}

impl <'a, T: ?Sized + Hash> Hash for DeepRef<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state)
    }
}
