use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use bdk::database::MemoryDatabase;

#[derive(Clone)]
pub struct AppWallet(Rc<RefCell<bdk::Wallet<MemoryDatabase>>>);

impl AppWallet {
    pub fn borrow(&self) -> Ref<bdk::Wallet<MemoryDatabase>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<bdk::Wallet<MemoryDatabase>> {
        self.0.borrow_mut()
    }
}

impl PartialEq for AppWallet {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}
