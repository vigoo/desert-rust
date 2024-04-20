use crate::serializer::{StoreRefResult, StoreStringResult};
use crate::{RefId, StringId};
use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use std::any::Any;

#[derive(Default)]
pub struct State {
    strings_by_id: HashMap<StringId, String>,
    ids_by_string: HashMap<String, StringId>,
    last_string_id: StringId,
    refs_by_id: HashMap<RefId, *const dyn Any>,
    ids_by_ref: HashMap<*const dyn Any, RefId>,
    last_ref_id: RefId,
}

impl State {
    pub fn store_string(&mut self, value: String) -> StoreStringResult {
        match self.ids_by_string.entry(value) {
            Entry::Occupied(entry) => StoreStringResult::StringAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                self.last_string_id.next();
                let id = self.last_string_id;
                self.strings_by_id.insert(id, entry.key().clone());
                let result = StoreStringResult::StringIsNew {
                    new_id: id,
                    value: entry.key().clone(),
                };
                entry.insert(id);
                result
            }
        }
    }

    pub fn store_ref(&mut self, value: &impl Any) -> StoreRefResult {
        match self.ids_by_ref.entry(value) {
            Entry::Occupied(entry) => StoreRefResult::RefAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                self.last_ref_id.next();
                let id = self.last_ref_id;
                self.refs_by_id.insert(id, value);
                let result = StoreRefResult::RefIsNew { new_id: id, value };
                entry.insert(id);
                result
            }
        }
    }

    pub fn get_string_by_id(&self, id: StringId) -> Option<&str> {
        self.strings_by_id.get(&id).map(|s| s.as_str())
    }

    pub fn get_ref_by_id(&self, id: RefId) -> Option<&dyn Any> {
        let ptr = self.refs_by_id.get(&id);
        match ptr {
            Some(ptr) => unsafe { ptr.as_ref() },
            None => None,
        }
    }
}
