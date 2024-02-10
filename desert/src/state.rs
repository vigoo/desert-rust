use crate::serializer::{StoreRefResult, StoreStringResult};
use crate::storable::StorableRef;
use crate::{RefId, StringId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct State {
    strings_by_id: HashMap<StringId, String>,
    ids_by_string: HashMap<String, StringId>,
    last_string_id: StringId,
    refs_by_id: HashMap<RefId, Rc<dyn StorableRef>>,
    ids_by_ref: HashMap<Rc<dyn StorableRef>, RefId>,
    last_ref_id: RefId,
}

impl State {
    pub fn store_string(&mut self, value: String) -> StoreStringResult {
        match self.ids_by_string.entry(value) {
            Entry::Occupied(entry) => StoreStringResult::StringAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                let id = self.last_string_id;
                self.last_string_id.next();
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

    pub fn store_ref(&mut self, value: Rc<dyn StorableRef>) -> StoreRefResult {
        match self.ids_by_ref.entry(value.clone()) {
            Entry::Occupied(entry) => StoreRefResult::RefAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                let id = self.last_ref_id;
                self.last_ref_id.next();
                self.refs_by_id.insert(id, value.clone());
                let result = StoreRefResult::RefIsNew { new_id: id, value };
                entry.insert(id);
                result
            }
        }
    }

    pub fn get_string_by_id(&self, id: StringId) -> Option<&str> {
        self.strings_by_id.get(&id).map(|s| s.as_str())
    }

    pub fn get_ref_by_id(&self, id: RefId) -> Option<Rc<dyn StorableRef>> {
        self.refs_by_id.get(&id).cloned()
    }
}
