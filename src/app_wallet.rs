use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use bdk::{
    bitcoin::Network, blockchain::EsploraBlockchain, database::MemoryDatabase,
    descriptor::IntoWalletDescriptor,
};

const BLOCKSTREAM_URL: &'static str = "https://blockstream.info/testnet/api";
const RAJ_URL: &'static str = "http://192.168.1.190:3002";

#[derive(Clone)]
pub struct AppWallet(pub Rc<RefCell<(bdk::Wallet<MemoryDatabase>, EsploraBlockchain)>>);

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
        let esplora = EsploraBlockchain::new(BLOCKSTREAM_URL, 20);
        Ok(Self(Rc::new(RefCell::new((wallet, esplora)))))
    }

    pub fn borrow(&self) -> Ref<(bdk::Wallet<MemoryDatabase>, EsploraBlockchain)> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<(bdk::Wallet<MemoryDatabase>, EsploraBlockchain)> {
        self.0.borrow_mut()
    }
}

impl PartialEq for AppWallet {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}
