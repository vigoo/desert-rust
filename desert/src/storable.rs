use dyn_eq::DynEq;
use dyn_hash::DynHash;
use std::any::Any;
use std::hash::Hash;

dyn_eq::eq_trait_object!(StorableRef);
dyn_hash::hash_trait_object!(StorableRef);

pub trait StorableRef: DynEq + DynHash {
    fn get(&self) -> &dyn Any;
}

impl<T: Any + Hash + Eq + 'static> StorableRef for T {
    fn get(&self) -> &dyn Any {
        self
    }
}
