extern crate interact;

use interact::Interact;
use interact_prompt::{LocalRegistry, Settings};
use std::{cell::RefCell, rc::Rc};

#[derive(Interact)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Interact)]
struct State {
    maybe_point: Option<Point>,
    complex: ((((usize, usize), u32, (u32, (u32,))), u32), u32),
    behind_rc: Rc<RefCell<u32>>,
    behind_rc2: Rc<RefCell<u32>>,
}

fn main() -> Result<(), interact_prompt::PromptError> {
    let rc = Rc::new(RefCell::new(3));
    let state = State {
        maybe_point: Some(Point { x: 3, y: 3 }),
        complex: ((((0, 0), 0, (0, (0,))), 0), 0),
        behind_rc: rc.clone(),
        behind_rc2: rc,
    };

    LocalRegistry::insert("state", Box::new(state));
    interact_prompt::direct(Settings::default(), ())?;
    Ok(())
}
