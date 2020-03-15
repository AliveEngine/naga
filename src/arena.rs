use std::{
    fmt,
    hash,
    marker::PhantomData,
};

/// An unique index in the arena array that a handle points to.
///
/// This type is independent of `spirv::Word`. `spirv::Word` is used in data
/// representation. It holds a SPIR-V and refers to that instruction. In
/// structured representation, we use Handle to refer to an SPIR-V instruction.
/// Index is an implementation detail to Handle.
type Index = u32;

/// A strongly typed reference to a SPIR-V element.
pub struct Handle<T> {
    index: Index,
    marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            index: self.index,
            marker: self.marker,
        }
    }
}
impl<T> Copy for Handle<T> {}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Handle<T> {}
impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Handle({})", self.index)
    }
}
impl<T> hash::Hash for Handle<T> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher)
    }
}

impl<T> Handle<T> {
    #[cfg(test)]
    pub const DUMMY: Self = Handle {
        index: !0,
        marker: PhantomData,
    };

    pub(crate) fn new(index: Index) -> Self {
        Handle {
            index,
            marker: PhantomData,
        }
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }
}

/// An arena holding some kind of component (e.g., type, constant,
/// instruction, etc.) that can be referenced.
#[derive(Debug)]
pub struct Arena<T> {
    /// Values of this arena.
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            data: Vec::new(),
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Handle<T>, &'a T)> {
        self.data
            .iter()
            .enumerate()
            .map(|(i, v)| (Handle::new(i as Index), v))
    }

    /// Adds a new value to the arena, returning a typed handle.
    ///
    /// The value is not linked to any SPIR-V module.
    pub fn append(&mut self, value: T) -> Handle<T> {
        let index = self.data.len() as Index;
        self.data.push(value);
        Handle::new(index)
    }

    /// Adds a value with a check for uniqueness: returns a handle pointing to
    /// an existing element if its value matches the given one, or adds a new
    /// element otherwise.
    pub fn fetch_or_append(&mut self, value: T) -> Handle<T> where T: PartialEq {
        if let Some(index) = self.data.iter().position(|d| d == &value) {
            Handle::new(index as Index)
        } else {
            self.append(value)
        }
    }
}

impl<T> std::ops::Index<Handle<T>> for Arena<T> {
    type Output = T;
    fn index(&self, handle: Handle<T>) -> &T {
        &self.data[handle.index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_non_unique() {
        let mut arena: Arena<f64> = Arena::new();
        let t1 = arena.append(0.0);
        let t2 = arena.append(0.0);
        assert!(t1 != t2);
        assert!(arena[t1] == arena[t2]);
    }

    #[test]
    fn append_unique() {
        let mut arena: Arena<f64> = Arena::new();
        let t1 = arena.append(std::f64::NAN);
        let t2 = arena.append(std::f64::NAN);
        assert!(t1 != t2);
        assert!(arena[t1] != arena[t2]);
    }

    #[test]
    fn fetch_or_append_non_unique() {
        let mut arena: Arena<f64> = Arena::new();
        let t1 = arena.fetch_or_append(0.0);
        let t2 = arena.fetch_or_append(0.0);
        assert!(t1 == t2);
        assert!(arena[t1] == arena[t2])
    }

    #[test]
    fn fetch_or_append_unique() {
        let mut arena: Arena<f64> = Arena::new();
        let t1 = arena.fetch_or_append(std::f64::NAN);
        let t2 = arena.fetch_or_append(std::f64::NAN);
        assert!(t1 != t2);
        assert!(arena[t1] != arena[t2]);
    }
}