extern crate interact;
extern crate structopt_derive;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use interact::Interact;
use interact_prompt::{LocalRegistry, SendRegistry, Settings};

#[derive(Interact)]
pub struct Chain {
    value: u32,
    nest: Option<Rc<RefCell<Chain>>>,
}

#[derive(Interact)]
pub struct Global {
    config: Config,
    cother: u32,
    items: BTreeMap<String, ItemState>,
    items_b: BTreeMap<((u32, String), u32), String>,
}

#[derive(Interact)]
pub struct Local {
    chain: Chain,
    loop_chain: Chain,
}

#[derive(Interact)]
struct Config {
    field: u32,
    sub_struct: SubConfig,
}

#[derive(Interact)]
struct SimpleStruct {
    field: u32,
    another: u32,
}

#[derive(Interact)]
#[interact(mut_fn(boo(a, bla)))]
#[interact(immut_fn(tes(a, x)))]
struct SubConfig {
    sub_a: u32,
    sub_b: u64,
    simple: SimpleStruct,
    sub_c: ((((usize, usize), u32, (u32, (u32,))), u32), u32),
    root_item: ItemState,
}

impl SubConfig {
    fn tes(&self, _a: u32, _b: u32) {
        println!("sub_a: {}", self.sub_a);
    }

    fn boo(&mut self, a: u32, bla: u32) -> u32 {
        self.sub_a = a;
        self.sub_b = bla.into();
        println!("Modified!");
        a + bla
    }
}

impl Global {
    fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            items_b: {
                let mut h = BTreeMap::new();
                h.insert(((10, "X".to_owned()), 3), "Y".to_owned());
                h
            },
            cother: 1,
            config: Config {
                field: 42,
                sub_struct: SubConfig {
                    simple: SimpleStruct {
                        field: 3,
                        another: 123,
                    },
                    sub_a: 128,
                    sub_c: ((((0, 0), 1, (1, (1,))), 1), 1),
                    sub_b: 99,
                    root_item: ItemState::new(HashMap::new()),
                },
            },
        }
    }

    fn populate(&mut self) {
        let mut hm = HashMap::new();
        hm.insert(3, "String B".to_owned());
        hm.insert(4, "String A".to_owned());
        self.items.insert("Item A".to_owned(), ItemState::new(hm));

        let mut hm = HashMap::new();
        hm.insert(31, "String C".to_owned());
        hm.insert(41, "String D".to_owned());
        hm.insert(51, "String E".to_owned());
        self.items.insert("Item X".to_owned(), ItemState::new(hm));
        self.items
            .insert("Item X2".to_owned(), ItemState::new(HashMap::new()));
        self.items
            .insert("Item X3".to_owned(), ItemState::new(HashMap::new()));
        self.items
            .insert("Item X4".to_owned(), ItemState::new(HashMap::new()));
        self.items
            .insert("Item X5".to_owned(), ItemState::new(HashMap::new()));

        let mut hm = HashMap::new();
        hm.insert(31, "String C".to_owned());
        hm.insert(41, "String: D".to_owned());
        hm.insert(44, "String: D".to_owned());
        hm.insert(45, "String: a longer string".to_owned());
        hm.insert(56, "String E".to_owned());
        hm.insert(57, "Lorem ipsum dolor sit amet, tation semper id vel, no officiis petentium expetenda qui. Mel odio accommodare ea. Ei detraxit suscipiantur cum, cum dico tantas impedit at. Ius delenit democritum omittantur ea, ut aeque ubique deterruisset sea.".to_owned());
        self.items.insert("Item Y".to_owned(), ItemState::new(hm));
    }
}

impl Local {
    fn new() -> Self {
        let node = Rc::new(RefCell::new(Chain {
            value: 2,
            nest: Some(Rc::new(RefCell::new(Chain {
                value: 3,
                nest: None,
            }))),
        }));

        node.borrow_mut().nest.as_ref().unwrap().borrow_mut().nest = Some(node.clone());

        Self {
            chain: Chain {
                value: 1,
                nest: Some(Rc::new(RefCell::new(Chain {
                    value: 2,
                    nest: Some(Rc::new(RefCell::new(Chain {
                        value: 3,
                        nest: None,
                    }))),
                }))),
            },
            loop_chain: Chain {
                value: 1,
                nest: Some(node),
            },
        }
    }
}

// #[derive(Interact)]
// enum Variant {
//     WithUnit,
//     WithTuple(u64, String),
//     WithFields {
//         some_str: String,
//         other_field: u32,
//     },
// }

#[derive(Interact)]
struct UnnamedFields(String, u32);

#[derive(Interact)]
struct UnitStruct;

#[derive(Interact)]
pub struct ItemState {
    unit: UnitStruct,
    fields: UnnamedFields,
    opt: Option<(u32, bool, i16, char)>,
    sub_items: HashMap<u32, String>,
    test_1: EnumTest,
    test_2: EnumTest,
    test_3: EnumTest,
    vec: Vec<u32>,
}

#[derive(Interact)]
pub enum EnumTest {
    Unit,
    Named { a: u32, b: u32 },
    Unnamed(u32),
}

#[derive(Interact)]
pub struct Dups {
    instant: std::time::Instant,
    a: Arc<u32>,
    b: Arc<u32>,
    c: Arc<u32>,
    d: Arc<u32>,
    e: Arc<u32>,
    f: Arc<u32>,
}

impl ItemState {
    fn new(sub_items: HashMap<u32, String>) -> Self {
        Self {
            unit: UnitStruct,
            opt: Some((32, false, 20, 'a')),
            fields: UnnamedFields("test".to_owned(), 42),
            sub_items,
            test_1: EnumTest::Unit,
            test_2: EnumTest::Named { a: 3, b: 5 },
            test_3: EnumTest::Unnamed(10),
            vec: vec![3, 4, 5],
        }
    }
}

use structopt::clap::AppSettings;
use structopt::StructOpt;
#[derive(StructOpt, Debug)]
#[structopt(raw(
    global_settings = "&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]"
))]
pub struct Opt {
    #[structopt(short = "i", long = "initial-command")]
    initial_command: Option<String>,

    #[structopt(short = "h", long = "history-file")]
    history_file: Option<String>,
}

fn main() -> Result<(), interact_prompt::PromptError> {
    let global = Arc::new(Mutex::new(Global::new()));
    let local = Local::new();

    {
        let mut global = global.lock().unwrap();
        global.populate();
    }

    SendRegistry::insert("global", Box::new(global.clone()));
    LocalRegistry::insert("local", Box::new(Mutex::new(RefCell::new(local))));

    let one = Arc::new(41u32);
    let two = Arc::new(52u32);
    let three = Arc::new(63u32);
    let dups = Arc::new(Mutex::new(Dups {
        instant: std::time::Instant::now(),
        a: three.clone(),
        b: two.clone(),
        c: two.clone(),
        d: three.clone(),
        e: three.clone(),
        f: one.clone(),
    }));
    SendRegistry::insert("dups", Box::new(dups));

    let Opt {
        history_file,
        initial_command,
    } = Opt::from_args();

    interact_prompt::direct(
        Settings {
            initial_command,
            history_file,
        },
        (),
    )?;
    Ok(())
}
