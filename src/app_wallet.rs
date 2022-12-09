use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use bdk::{bitcoin::Network, database::MemoryDatabase, descriptor::IntoWalletDescriptor};

#[derive(Clone)]
pub struct AppWallet(Rc<RefCell<bdk::Wallet<MemoryDatabase>>>);

impl AppWallet {
    pub fn new<E: IntoWalletDescriptor>(
        descriptor: E,
        change_descriptor: Option<E>,
        network: Network,
    ) -> Result<Self, bdk::Error> {
        let wallet = bdk::Wallet::new(
            descriptor,
            change_descriptor,
            network,
            MemoryDatabase::new(),
        )?;
        Ok(Self(Rc::new(RefCell::new(wallet))))
    }

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
