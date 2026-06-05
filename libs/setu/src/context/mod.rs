mod sorted_map;
mod store;

use std::{cell::RefCell, net::SocketAddr, rc::Rc};

pub use store::Store;

use crate::Timeout;

pub struct Context {
    pub state: Rc<State>,
    pub timeout: Option<Timeout>,
}

pub struct State {
    pub addr: SocketAddr,
    pub state: RefCell<Store>,
}

impl State {
    pub fn new(addr: SocketAddr) -> Rc<Self> {
        Rc::new(State {
            addr,
            state: RefCell::new(Store::new()),
        })
    }
}

