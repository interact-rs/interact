#![allow(dead_code, unused_imports, unused_mut, unused)] // REMOVE ME
extern crate interact;

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

#[macro_use]
extern crate interact_derive;
use pretty_assertions::assert_eq;

/// Cover private Mutex derive
mod mutex {
    use interact::access::{Access, ReflectDirect};
    use interact::climber::{ClimbError, Climber};
    use interact::deser::{self, Tracker};
    use interact::{Deser, NodeTree, Reflector};

    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::sync::Arc;

    pub struct PseudoMutex<T> {
        t: T,
    }

    impl<T> PseudoMutex<T> {
        fn new(t: T) -> Self {
            PseudoMutex { t }
        }
    }

    struct Guard(());

    impl<T> PseudoMutex<T> {
        fn lock(&self) -> Guard {
            Guard(())
        }
    }

    impl Deref for Guard {
        type Target = ();

        fn deref(&self) -> &() {
            &self.0
        }
    }

    impl DerefMut for Guard {
        fn deref_mut(&mut self) -> &mut () {
            &mut self.0
        }
    }

    impl<T> ReflectDirect for PseudoMutex<T>
    where
        T: Access,
    {
        fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
            let locked = self.lock();
            Reflector::reflect(reflector, &*locked)
        }

        fn immut_climber<'a>(
            &self,
            climber: &mut Climber<'a>,
        ) -> Result<Option<NodeTree>, ClimbError> {
            let save = climber.clone();
            let retval = {
                let locked = self.lock();
                climber.general_access_immut(&*locked).map(Some)
            };

            if let Err(ClimbError::NeedMutPath) = &retval {
                *climber = save;
                let mut locked = self.lock();
                climber.general_access_mut(&mut *locked).map(Some)
            } else {
                retval
            }
        }

        fn mut_climber<'a>(
            &mut self,
            climber: &mut Climber<'a>,
        ) -> Result<Option<NodeTree>, ClimbError> {
            let mut locked = self.lock();
            climber.general_access_mut(&mut *locked).map(Some)
        }
    }

    impl<T> Deser for PseudoMutex<T>
    where
        T: Deser,
    {
        fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> deser::Result<Self> {
            Ok(PseudoMutex::new(T::deser(tracker)?))
        }
    }
}

/// Cover `derive` variaty

#[derive(Clone, Interact)]
struct Foo {
    a: u32,
    b: u32,
}

#[derive(Clone, Interact)]
struct Foo2(u32, u32);

#[derive(Clone, Interact)]
struct State {
    u: u32,
    opt: Option<Foo>,
    op2: Option<u32>,
    foo: Foo,
    test: Arc<Mutex<Foo>>,
    v: Vec<u32>,
    m: BTreeMap<u32, u32>,
}

macro_rules! verify {
    ($self:expr, $e:expr => $result:tt) => {
        let e = $e;
        let str_e = format!("{:?}", e);

        if $self.check {
            if str_e != $result {
                println!("");
                println!("Failed:");
                println!("");
                println!("verify!(self, {} => {:?});", stringify!($e), str_e);
                println!("");

                let result = std::panic::catch_unwind(|| {
                    assert_eq!(str_e, $result);
                });

                $self.count += 1;
            }
        } else {
            println!("verify!(self, {} => {:?});", stringify!($e), str_e);
        }
    };
}

struct Context {
    count: usize,
    check: bool,
}

#[rustfmt::skip]
impl Context {
    fn main(&mut self) {
        let mut state = State {
            u: 10,
            opt: Some(Foo { a: 10, b: 100 }),
            op2: None,
            foo: Foo { a: 2, b: 4 },
            test: Arc::new(Mutex::new(Foo { a: 10, b: 100 })),
            v: vec![1, 2, 3],
            m: {
                let mut bm = BTreeMap::new();
                bm.insert(3, 4);
                bm.insert(7, 8);
                bm
            },
        };

        let mut root = interact::RootSend::new();
        root.owned.insert("state", Box::new(state));
        let mut root = root.as_root();

        println!("");

        verify!(self, root.access("state.u") => "(Ok(NodeTree { info: Leaf(\"10\"), meta: Some(Wrap(1)), size: 3 }), Assist { valid: 7, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("state.nonexist") => "(Err(UnexpectedToken), Assist { valid: 5, pending: 1, pending_special: 0, next_options: Avail(1, []) })");
        verify!(self, root.access("state.m") => "(Ok(NodeTree { info: Named(NodeTree { info: Leaf(\"BTreeMap\"), meta: None, size: 9 }, NodeTree { info: Grouped(\'{\', NodeTree { info: Delimited(\',\', [NodeTree { info: Tuple(NodeTree { info: Leaf(\"3\"), meta: Some(Wrap(1)), size: 2 }, \":\", NodeTree { info: Leaf(\"4\"), meta: Some(Wrap(1)), size: 2 }), meta: None, size: 8 }, NodeTree { info: Tuple(NodeTree { info: Leaf(\"7\"), meta: Some(Wrap(1)), size: 2 }, \":\", NodeTree { info: Leaf(\"8\"), meta: Some(Wrap(1)), size: 2 }), meta: None, size: 8 }]), meta: None, size: 21 }, \'}\'), meta: None, size: 24 }), meta: Some(Wrap(1)), size: 34 }), Assist { valid: 7, pending: 0, pending_special: 0, next_options: Avail(0, [\".len(\", \"[\"]) })");
        verify!(self, root.access("state.m[3]") => "(Ok(NodeTree { info: Leaf(\"4\"), meta: Some(Wrap(1)), size: 2 }), Assist { valid: 10, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("state.m.len()") => "(Ok(NodeTree { info: Leaf(\"2\"), meta: Some(Wrap(1)), size: 2 }), Assist { valid: 13, pending: 0, pending_special: 0, next_options: Avail(0, []) })");

        if self.count > 0 {
            panic!("A total of {} verification tests failed", self.count)
        }
    }
}

#[test]
fn main() {
    let mut context = Context {
        count: 0,
        check: false,
    };
    context.main();

    context.check = true;
    context.main();
}
