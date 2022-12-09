use gloo_storage::{LocalStorage, Storage};

use super::State;

const STORAGE_KEY: &str = "KEYMAN_STATE";

pub fn save(state: &State) {
    LocalStorage::set(STORAGE_KEY, state).unwrap()
}

pub fn load() -> Option<State> {
    LocalStorage::get(STORAGE_KEY).ok()
}
