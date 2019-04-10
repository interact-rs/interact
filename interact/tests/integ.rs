extern crate interact;

use pretty_assertions::assert_eq;
mod common;
use common::{Basic, Complex, LocalRcLoop, Rand};

struct Context {
    count: usize,
    check: bool,
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
                println!("verify!(self, {} => {:?});", stringify!($e), $result);
                println!("");

                let _ = std::panic::catch_unwind(|| {
                    assert_eq!(str_e, $result);
                });

                println!("Correct this by having:");
                println!("");
                println!("verify!(self, {} => {:?});", stringify!($e), str_e);
                println!("");

                $self.count += 1;
            }
        } else {
            println!("verify!(self, {} => {:?});", stringify!($e), str_e);
        }
    };
}

#[test]
fn main() {
    let mut context = Context {
        count: 0,
        check: true,
    };
    context.main();

    if context.count > 0 {
        {
            println!();
            println!("Expected test manifest:");
            println!();
            let mut context = Context {
                count: 0,
                check: false,
            };
            context.main();
            println!();
        }

        panic!("A total of {} verification tests failed", context.count)
    }
}

#[rustfmt::skip]
impl Context {
    fn main(&mut self) {
        let mut root = interact::RootSend::new();
        let mut root_local = interact::RootLocal::new();
        let seed = 42;
        let mut rng: rand::StdRng = rand::SeedableRng::seed_from_u64(seed);

        root.owned.insert("complex", Box::new(Complex::new_random(&mut rng)));
        root.owned.insert("basic", Box::new(Basic::new_random(&mut rng)));
        root_local.owned.insert("rc_loops", Box::new(LocalRcLoop::new_random(&mut rng)));

        let mut root = interact::Root {
            send: Some(&mut root),
            local: Some(&mut root_local),
        };

        // Check for a non-existing root key

        verify!(self, root.access("not_existing") => "(Err(MissingStartComponent), Assist { valid: 0, pending: 0, pending_special: 0, next_options: Avail(0, []) })");

        // Check for a basic read query

        verify!(self, root.access("basic.u_16") => "(Ok(NodeTree { info: Leaf(\"50158\"), meta: Some(Wrap(1)), size: 6 }), Assist { valid: 10, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("basic.u_") => "(Err(UnexpectedToken), Assist { valid: 5, pending: 3, pending_special: 0, next_options: Avail(1, [\"u_s\", \"u_64\", \"u_32\", \"u_16\", \"u_8\"]) })");

        // Basic assignment check

        verify!(self, root.access("basic.u_64 = 1234") => "(Ok(NodeTree { info: Leaf(\"\"), meta: None, size: 1 }), Assist { valid: 17, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("basic.u_64") => "(Ok(NodeTree { info: Leaf(\"1234\"), meta: Some(Wrap(1)), size: 5 }), Assist { valid: 10, pending: 0, pending_special: 0, next_options: Avail(0, []) })");

        // Token parsing error

        verify!(self, root.access("state.complex.0.0.0.0 = 100000000000000000001") => "(Err(TokenError(IntError(ParseIntError { kind: Overflow }))), Assist { valid: 0, pending: 0, pending_special: 0, next_options: Avail(0, []) })");

        // Verify calling immutable methods from prompt

        verify!(self, root.access("complex.tuple.0.0 = 3") => "(Ok(NodeTree { info: Leaf(\"\"), meta: None, size: 1 }), Assist { valid: 21, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.tuple_1.0 = 3") => "(Ok(NodeTree { info: Leaf(\"\"), meta: None, size: 1 }), Assist { valid: 21, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.check()") => "(Ok(NodeTree { info: Leaf(\"true\"), meta: Some(Wrap(1)), size: 5 }), Assist { valid: 15, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.tuple_1.0 = 4") => "(Ok(NodeTree { info: Leaf(\"\"), meta: None, size: 1 }), Assist { valid: 21, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.check()") => "(Ok(NodeTree { info: Leaf(\"false\"), meta: Some(Wrap(1)), size: 6 }), Assist { valid: 15, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.add(3)") => "(Ok(NodeTree { info: Leaf(\"()\"), meta: Some(Wrap(1)), size: 3 }), Assist { valid: 14, pending: 0, pending_special: 0, next_options: Avail(0, []) })");
        verify!(self, root.access("complex.tuple_1.0 = 7") => "(Ok(NodeTree { info: Leaf(\"\"), meta: None, size: 1 }), Assist { valid: 21, pending: 0, pending_special: 0, next_options: Avail(0, []) })");

        // TODO: add more comparision tests
    }
}
