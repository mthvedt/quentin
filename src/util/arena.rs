use once_cell::unsync::OnceCell;
use typed_arena::Arena;

/// An arena that allocates references and forward references.
pub struct ForwardArena<'a, T: 'a> {
    // TODO: hide
    alloc: Arena<T>,
    ref_alloc: Arena<OnceCell<&'a T>>,
}

impl<'a, T: 'a> ForwardArena<'a, T> {
    pub fn new() -> Self {
        ForwardArena {
            alloc: Arena::new(),
            ref_alloc: Arena::new(),
        }
    }

    pub fn forward(&'a self) -> ForwardCell<'a, T> {
        ForwardCell {
            arena: self,
            cell: self.ref_alloc.alloc(OnceCell::new()),
        }
    }
}

pub struct ForwardRef<'a, T: 'a>(&'a OnceCell<&'a T>);

impl<'a, T: 'a> ForwardRef<'a, T> {
    pub fn get(&self) -> Option<&'a T> {
        self.0.get().map(|t| *t)
    }
}

impl<'a, T: 'a> Clone for ForwardRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: 'a> Copy for ForwardRef<'a, T> {}

pub struct ForwardCell<'a, T: 'a> {
    arena: &'a ForwardArena<'a, T>,
    cell: &'a OnceCell<&'a T>,
}

impl<'a, T: 'a> ForwardCell<'a, T> {
    pub fn borrow(&self) -> ForwardRef<'a, T> {
        ForwardRef(self.cell)
    }

    pub fn set(self, t: T) -> ForwardRef<'a, T> {
        // since only one ForwardCell to a ref is allowed, this should only happen once
        self.cell.set(self.arena.alloc.alloc(t)).ok();
        ForwardRef(self.cell)
    }
}
