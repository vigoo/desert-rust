use crate::serializer::{StoreRefResult, StoreStringResult};
use crate::{RefId, StringId};
use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use std::any::Any;
use std::cell::OnceCell;

#[derive(Default)]
struct StringsState {
    strings_by_id: HashMap<StringId, String>,
    ids_by_string: HashMap<String, StringId>,
    last_string_id: StringId,
}

#[derive(Default)]
struct RefsState {
    /// Deserialization: values that later references resolve to, owned by the state.
    values_by_id: HashMap<RefId, Box<dyn Any>>,
    /// Serialization: addresses of values already written. Only compared, never dereferenced.
    ids_by_ref: HashMap<*const dyn Any, RefId>,
    last_ref_id: RefId,
}

#[derive(Default)]
pub struct State {
    strings: OnceCell<StringsState>,
    refs: OnceCell<RefsState>,
}

impl State {
    pub fn store_string(&mut self, value: String) -> StoreStringResult {
        self.strings.get_or_init(StringsState::default);
        let strings = unsafe { self.strings.get_mut().unwrap_unchecked() };
        match strings.ids_by_string.entry(value) {
            Entry::Occupied(entry) => StoreStringResult::StringAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                strings.last_string_id.next();
                let id = strings.last_string_id;
                strings.strings_by_id.insert(id, entry.key().clone());
                let result = StoreStringResult::StringIsNew {
                    new_id: id,
                    value: entry.key().clone(),
                };
                entry.insert(id);
                result
            }
        }
    }

    /// Assigns the next reference id to a value written during serialization, or returns the id
    /// it already has. Values are identified by address, so they must stay alive while the
    /// serialization runs.
    pub fn store_ref_identity(&mut self, value: &impl Any) -> StoreRefResult {
        self.refs.get_or_init(RefsState::default);
        let refs = unsafe { self.refs.get_mut().unwrap_unchecked() };
        match refs.ids_by_ref.entry(value) {
            Entry::Occupied(entry) => StoreRefResult::RefAlreadyStored { id: *entry.get() },
            Entry::Vacant(entry) => {
                refs.last_ref_id.next();
                let id = refs.last_ref_id;
                let result = StoreRefResult::RefIsNew { new_id: id, value };
                entry.insert(id);
                result
            }
        }
    }

    /// Stores a deserialized value under the next reference id, so that later references to it
    /// resolve through `get_ref_by_id`. The state keeps the value until the deserialization ends,
    /// so store a cheap handle to shared data, such as a clone of an `Rc`.
    pub fn store_ref<T: Any>(&mut self, value: T) -> RefId {
        self.refs.get_or_init(RefsState::default);
        let refs = unsafe { self.refs.get_mut().unwrap_unchecked() };
        refs.last_ref_id.next();
        let id = refs.last_ref_id;
        refs.values_by_id.insert(id, Box::new(value));
        id
    }

    pub fn get_string_by_id(&self, id: StringId) -> Option<&str> {
        self.strings
            .get_or_init(StringsState::default)
            .strings_by_id
            .get(&id)
            .map(|s| s.as_str())
    }

    pub fn get_ref_by_id(&self, id: RefId) -> Option<&dyn Any> {
        self.refs
            .get_or_init(RefsState::default)
            .values_by_id
            .get(&id)
            .map(|value| &**value)
    }
}
