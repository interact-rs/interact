//! Interact Prompt registry for accessible state.

use std::cell::RefCell;
use std::sync::Mutex;

use interact::{Access, Root, RootLocal, RootSend};

/// The `Send` Registry manages state roots of the whole process.
pub struct SendRegistry {
    root: Mutex<RootSend>,
}

lazy_static! {
    static ref REGISTRY: SendRegistry = {
        SendRegistry {
            root: Mutex::new(RootSend::new()),
        }
    };
}

impl SendRegistry {
    /// Insert new states into the root.
    pub fn insert(string: &'static str, item: Box<dyn Access + Send>) {
        let mut root = REGISTRY.root.lock().unwrap();

        root.owned.insert(string, item);
    }

    #[doc(hidden)]
    pub(crate) fn with_root<F, R>(f: F) -> R
    where
        F: FnOnce(&mut RootSend) -> R,
    {
        let mut root = REGISTRY.root.lock().unwrap();
        f(&mut *root)
    }
}

/// The Local Registry manages state roots of per-thread states.
pub struct LocalRegistry {
    root: RootLocal,
}

thread_local! {
    #[doc(hidden)]
    pub static LOCAL_REGISTRY: RefCell<LocalRegistry> = {
        RefCell::new(LocalRegistry {
            root: RootLocal::new(),
        })
    };
}

impl LocalRegistry {
    /// Insert new states into the root.
    pub fn insert(string: &'static str, item: Box<dyn Access>) {
        LOCAL_REGISTRY.with(|reg| {
            reg.borrow_mut().root.owned.insert(string, item);
        });
    }
}

#[doc(hidden)]
pub(crate) fn with_root<F, R>(f: F) -> R
where
    F: FnOnce(&mut Root) -> R,
{
    LOCAL_REGISTRY.with(|local_reg| {
        SendRegistry::with_root(|send_reg| {
            let mut local_reg = local_reg.borrow_mut();
            let mut root = Root {
                send: Some(send_reg),
                local: Some(&mut local_reg.root),
            };
            f(&mut root)
        })
    })
}
