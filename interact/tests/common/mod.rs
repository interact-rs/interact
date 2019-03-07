#![allow(unused)]

/// Here we define types and data to be used in tests and examples.
pub mod pseudo_mutex;
mod random;
use pseudo_mutex::PseudoMutex;
pub use random::Rand;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use interact::Interact;
use rand::Rng;

#[derive(Interact)]
pub struct Basic {
    u_s: usize,
    is: isize,
    u_64: u64,
    u_32: u32,
    u_16: u16,
    u_8: u8,
    bo: bool,
    st: String,
    ch: char,
    i_64: i64,
    i_32: i32,
    i_16: i16,
    i_8: u8,
    arr: [u8; 4],
    option_none: Option<u8>,
    option_some: Option<u8>,
    result_ok: Result<u8, u32>,
    result_err: Result<u8, u32>,
}

fn new_string_random<R: Rng>(rng: &mut R) -> String {
    const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789_";
    String::from_iter((0..20).map(|_| {
        CHARSET
            .chars()
            .nth(rng.gen::<usize>() % CHARSET.len())
            .unwrap()
    }))
}

impl Rand for Basic {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        Self {
            u_s: rng.gen(),
            is: rng.gen(),
            u_64: rng.gen(),
            u_32: rng.gen(),
            u_16: rng.gen(),
            u_8: rng.gen(),
            arr: [rng.gen(), rng.gen(), rng.gen(), rng.gen()],
            bo: rng.gen(),
            st: new_string_random(rng),
            ch: rng.gen(),
            i_64: rng.gen(),
            i_32: rng.gen(),
            i_16: rng.gen(),
            i_8: rng.gen(),
            option_none: None,
            option_some: Some(rng.gen()),
            result_ok: Ok(rng.gen()),
            result_err: Err(rng.gen()),
        }
    }
}

#[derive(Interact, Hash, Eq, PartialEq)]
pub struct Key {
    field_a: u64,
    field_b: u32,
}

impl Rand for Key {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        Self {
            field_a: rng.gen(),
            field_b: rng.gen(),
        }
    }
}

#[derive(Interact)]
pub enum EnumExample {
    VarUnit,
    VarUnnamed(u8, u32),
    VarNamed { a: u8, b: u16 },
}

#[derive(Interact)]
pub struct RefsAndLocks {
    arc_a: Arc<u32>,
    arc_b: Arc<Key>,
    arc_c: Arc<Key>,
    arc_d: Arc<u32>,
    arc_e: Arc<u32>,
    arc_f: Arc<u32>,
}

impl Rand for RefsAndLocks {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        let arc_a: Arc<u32> = Arc::new(Rand::new_random(rng));
        let arc_b = Arc::new(Key {
            field_a: Rand::new_random(rng),
            field_b: Rand::new_random(rng),
        });
        let arc_c = arc_b.clone();
        let arc_d = arc_a.clone();
        let arc_e = Arc::new(Rand::new_random(rng));
        let arc_f = Arc::new(Rand::new_random(rng));

        Self {
            arc_a,
            arc_b,
            arc_c,
            arc_d,
            arc_e,
            arc_f,
        }
    }
}

#[derive(Interact)]
struct UnnamedFields(String, u32);

#[derive(Interact)]
struct UnitStruct;

#[derive(Interact)]
#[interact(immut_fn(check()))]
#[interact(mut_fn(add(a)))]
pub struct Complex {
    simple: HashMap<u64, u32>,
    complex_key: HashMap<Key, u32>,
    map: BTreeMap<String, u32>,
    struct_unnamed: UnnamedFields,
    struct_unit: UnitStruct,
    enum_unit: EnumExample,
    enum_unnamed: EnumExample,
    enum_named: EnumExample,
    boxed: Box<EnumExample>,
    tuple: ((u32, EnumExample, (u8, u8)), i32),
    tuple_1: (u32,),
    vec: Vec<(u32, EnumExample)>,
    behind_mutex: Mutex<u32>,
    behind_pseudo_mutex: PseudoMutex<u64>,
    behind_arc_mutex: Arc<Mutex<u32>>,
    instant: Instant,
    refs: RefsAndLocks,
}

impl Complex {
    fn check(&self) -> bool {
        self.tuple.0 .0 == self.tuple_1.0
    }

    fn add(&mut self, a: u32) {
        self.vec[0].0 += a;
    }
}

impl Rand for Complex {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        let mut simple = HashMap::new();
        let mut complex_key = HashMap::new();
        let mut map = BTreeMap::new();

        for _ in 0..(2 + rng.gen::<u8>() / 32) {
            complex_key.insert(Rand::new_random(rng), Rand::new_random(rng));
        }
        for _ in 0..(2 + rng.gen::<u8>() / 32) {
            simple.insert(Rand::new_random(rng), Rand::new_random(rng));
        }
        for _ in 0..(2 + rng.gen::<u8>() / 32) {
            map.insert(new_string_random(rng), Rand::new_random(rng));
        }

        Self {
            simple,
            complex_key,
            map,
            struct_unit: UnitStruct,
            struct_unnamed: UnnamedFields(new_string_random(rng), Rand::new_random(rng)),
            enum_unit: EnumExample::VarUnit,
            enum_unnamed: EnumExample::VarUnnamed(Rand::new_random(rng), Rand::new_random(rng)),
            enum_named: EnumExample::VarNamed {
                a: Rand::new_random(rng),
                b: Rand::new_random(rng),
            },
            boxed: Box::new(EnumExample::VarUnit),
            tuple: (
                (
                    Rand::new_random(rng),
                    EnumExample::VarUnit,
                    (Rand::new_random(rng), Rand::new_random(rng)),
                ),
                Rand::new_random(rng),
            ),
            tuple_1: (0,),
            refs: Rand::new_random(rng),
            behind_arc_mutex: Arc::new(Mutex::new(Rand::new_random(rng))),
            behind_mutex: Mutex::new(Rand::new_random(rng)),
            behind_pseudo_mutex: PseudoMutex::new(Rand::new_random(rng)),
            vec: vec![
                (Rand::new_random(rng), EnumExample::VarUnit),
                (Rand::new_random(rng), EnumExample::VarNamed { a: 3, b: 4 }),
            ],
            instant: Instant::now(),
        }
    }
}

#[derive(Interact)]
pub struct Chain {
    value: u32,
    nest: Option<Rc<RefCell<Chain>>>,
}

#[derive(Interact)]
pub struct LocalRcLoop {
    chain: Chain,
    loop_chain: Chain,
}

impl Rand for LocalRcLoop {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        let node = Rc::new(RefCell::new(Chain {
            value: Rand::new_random(rng),
            nest: Some(Rc::new(RefCell::new(Chain {
                value: Rand::new_random(rng),
                nest: None,
            }))),
        }));

        node.borrow_mut().nest.as_ref().unwrap().borrow_mut().nest = Some(node.clone());

        Self {
            chain: Chain {
                value: Rand::new_random(rng),
                nest: Some(Rc::new(RefCell::new(Chain {
                    value: Rand::new_random(rng),
                    nest: Some(Rc::new(RefCell::new(Chain {
                        value: Rand::new_random(rng),
                        nest: None,
                    }))),
                }))),
            },
            loop_chain: Chain {
                value: Rand::new_random(rng),
                nest: Some(node),
            },
        }
    }
}

#[derive(Interact)]
pub struct LocalComplex {
    rc_loop: LocalRcLoop,
}
