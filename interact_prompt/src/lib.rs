//! Interact Prompt
//!
//! In high-level, to use Interact Prompt you need:
//!
//! 1) Deriving of `Interact` over types.
//! 2) Registration of state
//! 3) Spawning or invoking the Interact prompt.
//!
//! In pseudo code:
//!
//! ```ignore
//! extern crate interact;
//!
//! use interact::Interact;
//! use interact_prompt::{LocalRegistry, SendRegistry, Settings};
//!
//! #[derive(Interact)]
//! struct YourType {
//!     // ...
//! }
//!
//! // ...
//!
//! fn in_each_thread(rc: Rc<SomeState>) {
//!     // ... You would have some code to register your 'Rc's.
//!     LocalRegistry::insert("rc_state", Box::new(rc));
//!     // ...
//! }
//!
//! fn spawn_interact(arc: Arc<SomeOtherState>) {
//!     // On the global context you can register any object that is `Send`
//!     // and implements `Access` via #[derive(Interact)], this means `Arc` types,
//!     SendRegistry::insert("arc_state", Box::new(arc));
//!
//!     interact_prompt::spawn(Settings::default(), ());
//! }
//! ```
//!
//! NOTE: Currently only the `SendRegistry` is supported for the background `spawn` variant of
//! Interact. Supporting LocalRegistry is planned for the future.
//!

#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate interact;
extern crate rustyline;

use ansi_term::Color;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::Helper;
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline::validate::Validator;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::BTreeMap;
use std::thread;

use interact::{Assist, NextOptions, NodeTree};

mod print;
pub mod registry;
pub use crate::registry::{LocalRegistry, SendRegistry};

#[derive(Clone)]
pub struct Settings {
    pub history_file: Option<String>,
    pub initial_command: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_file: None,
            initial_command: None,
        }
    }
}

#[derive(Clone)]
pub enum Interaction {
    Line(String),
    CtrlC,
    CtrlD,
    Err,
}

#[derive(Clone)]
pub enum Response {
    Continue,
    Exit,
}

struct Commands {
    handlers: BTreeMap<&'static str, Box<dyn Command>>,
    def_handler: Box<dyn Command>,
}

fn print(elem: &NodeTree) {
    print::pretty_format(
        elem,
        &print::NodePrinterSettings {
            max_line_length: 120,
            indent_step: 4,
        },
    )
}

trait Command {
    fn handle(&self, commands: &Commands, params: Vec<String>);
    fn help(&self) -> &'static [&'static str];
    fn name(&self) -> &'static str;
    fn get_completions(&self, line: &str) -> Assist<String>;
}

struct Help;

impl Command for Help {
    fn handle(&self, commands: &Commands, _params: Vec<String>) {
        println!();
        println!("The following are the valid commands:");
        println!();

        for command in &commands.handlers {
            for line in command.1.help() {
                println!("      {}", line);
            }
        }

        println!();

        registry::with_root(|root| {
            println!("Possible nodes to evaluate from:");
            println!();
            for k in root.keys() {
                println!("      {}", k);
            }
            println!();
        })
    }

    fn help(&self) -> &'static [&'static str] {
        &[":help           Prints this help screen"]
    }
    fn name(&self) -> &'static str {
        ":help"
    }
    fn get_completions(&self, _line: &str) -> Assist<String> {
        Assist::default()
    }
}

struct Exit;

impl Command for Exit {
    fn handle(&self, _commands: &Commands, _params: Vec<String>) {
        std::process::exit(0);
    }

    fn help(&self) -> &'static [&'static str] {
        &[":exit           Terminate the program"]
    }
    fn name(&self) -> &'static str {
        ":exit"
    }
    fn get_completions(&self, _line: &str) -> Assist<String> {
        Assist::default()
    }
}

struct Access;

impl Command for Access {
    fn handle(&self, _commands: &Commands, params: Vec<String>) {
        let rest_of_string = params.join(" ");

        registry::with_root(|root| {
            let res = root.access(&rest_of_string).0;
            match res {
                Ok(read_value) => {
                    print(&read_value);
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        })
    }

    fn help(&self) -> &'static [&'static str] {
        &["<expr>          Access the value of expr"]
    }

    fn name(&self) -> &'static str {
        "<expr>"
    }

    fn get_completions(&self, line: &str) -> Assist<String> {
        registry::with_root(|root| root.probe(line).1)
    }
}

impl Commands {
    fn new() -> Self {
        let mut handlers = BTreeMap::new();

        for command in vec![
            Box::new(Help) as Box<dyn Command>,
            Box::new(Exit) as Box<dyn Command>,
        ]
        .into_iter()
        {
            handlers.insert(command.name(), command);
        }

        Commands {
            handlers,
            def_handler: Box::new(Access) as Box<dyn Command>,
        }
    }

    fn handle_cmd(&self, line: &str) {
        let mut params: Vec<String> = line.split(' ').map(|x| x.to_owned()).collect();
        if params == ["?"] {
            Help.handle(&self, params);
        } else if params != [""] {
            if let Some(command) = self.handlers.get(params[0].as_str()) {
                params.remove(0);
                (**command).handle(self, params);
            } else {
                self.def_handler.handle(self, params)
            }
        }
    }

    fn get_next_options(&self, line: &str, pos: usize) -> Assist<String> {
        if line == "?" || line == ":" {
            let mut assist = Assist::default();
            assist.pend_one();
            return assist;
        }

        let split: Vec<String> = line[..pos].split(' ').map(|x| x.to_owned()).collect();
        let prefix = split
            .iter()
            .filter(|x| x.as_str() != "")
            .nth(0)
            .cloned()
            .unwrap_or_else(|| String::from(""));

        let matching: Vec<_> = self
            .handlers
            .keys()
            .filter(|x| x.starts_with(&prefix))
            .map(|x| String::from(*x))
            .collect();
        if matching.len() == 1 {
            let match1 = &matching[0];

            if let Some(handler) = self.handlers.get(match1.as_str()) {
                let mut reconstruct = vec![];
                for i in split.iter() {
                    if i.trim() != "" && match1.starts_with(i) {
                        reconstruct.push(match1.clone());
                        break;
                    }
                    reconstruct.push(i.to_owned());
                }

                let reconstruct = reconstruct.join(" ");
                if reconstruct.starts_with(&line[..pos]) {
                    Assist::default()
                        .with_valid(pos)
                        .next_options(NextOptions::Avail(
                            0,
                            vec![String::from(&reconstruct[pos..])],
                        ))
                } else if line[..pos].starts_with(&reconstruct) {
                    let deeper = String::from(&line[reconstruct.len()..]);
                    let nospace = deeper
                        .chars()
                        .position(|c| c != ' ')
                        .unwrap_or_else(|| deeper.len());
                    let sub_access = (**handler).get_completions(&deeper[nospace..]);
                    sub_access.with_valid(reconstruct.len() + nospace)
                } else {
                    Assist::default()
                }
            } else {
                Access.get_completions(line)
            }
        } else if split != [""] {
            Access.get_completions(line)
        } else {
            Assist::default()
        }
    }
}

/// This trait defines an optional handler for prompt commands. This allows to
/// override the behavior of the handler for `()`.
pub trait Handler {
    fn receive_interaction(&self, intr: Interaction) -> Response {
        match intr {
            Interaction::Line(string) => {
                Commands::new().handle_cmd(&string);
            }
            Interaction::CtrlC | Interaction::CtrlD => {
                std::process::exit(0);
            }
            _ => {}
        }
        Response::Continue
    }
}

// InteractPromptHelper

struct InteractPromptHelper<'a, H>((), &'a H)
where
    H: 'a;

impl<'a, H> Completer for InteractPromptHelper<'a, H>
where
    H: 'a,
{
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<String>), ReadlineError> {
        let (valid, _, _, options) = Commands::new().get_next_options(line, pos).dismantle();
        Ok(options.into_position(valid))
    }
}

impl<'a, H> Hinter for InteractPromptHelper<'a, H> {
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let (valid, _, _, options) = Commands::new().get_next_options(line, pos).dismantle();
        let (from_pos, v) = options.into_position(valid);
        if v.len() == 1 {
            if from_pos < pos {
                let v0_len = v[0].len();
                Some(v[0][std::cmp::min(pos - from_pos, v0_len)..].to_owned())
            } else {
                Some(v[0].to_owned())
            }
        } else {
            None
        }
    }
}

impl<'a, H> Highlighter for InteractPromptHelper<'a, H> {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        let s = format!("{}", Color::Fixed(240).paint(hint));
        Owned(s.to_owned())
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let (valid, pending, pending_valid, _) =
            Commands::new().get_next_options(line, pos).dismantle();
        let yellow_cutoff = std::cmp::min(valid, line.len());
        let green_cutoff = std::cmp::min(valid + pending - pending_valid, line.len());
        let red_cutoff = std::cmp::min(valid + pending, line.len());

        let ok = &line[..yellow_cutoff].to_string();
        let pending = format!(
            "{}",
            Color::Yellow.paint(&line[yellow_cutoff..green_cutoff])
        );
        let pending_valid = format!(
            "{}",
            Color::Green.bold().paint(&line[green_cutoff..red_cutoff])
        );
        let err = format!("{}", Color::Red.paint(&line[red_cutoff..]));

        Owned(format!("{}{}{}{}", ok, pending, pending_valid, err))
    }

    fn highlight_char(&self, _grapheme: &str, _pos: usize) -> bool {
        false
    }
}

impl<'a, H> Helper for InteractPromptHelper<'a, H> {}

impl<'a, H> Validator for InteractPromptHelper<'a, H> {}

impl Handler for () {}

#[derive(Debug)]
pub enum PromptError {
    ReadLine(rustyline::error::ReadlineError),
}

/// Use the current thread for an interactive `Interact` prompt.
pub fn direct<H>(settings: Settings, handler: H) -> Result<(), PromptError>
where
    H: Handler
{
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let mut rl = Editor::with_config(config);

    let Settings {
        history_file,
        initial_command,
    } = settings;
    let h = InteractPromptHelper((), &handler);
    rl.set_helper(Some(h));

    println!("Rust `interact`, type '?' for more information");

    if let Some(history_file) = &history_file {
        rl.load_history(history_file)
            .map_err(PromptError::ReadLine)?;
    }

    match initial_command {
        None => {}
        Some(initial_command) => {
            println!("{}", initial_command);
            let response = handler.receive_interaction(Interaction::Line(initial_command));
            match response {
                Response::Exit => return Ok(()),
                Response::Continue => {}
            }
        }
    }

    loop {
        let prompt = format!("{} ", Color::Fixed(240).bold().paint(">>>"));
        let line = rl.readline(&prompt);

        let interaction = match line {
            Ok(line) => {
                rl.add_history_entry(&line);
                Interaction::Line(line)
            }
            Err(ReadlineError::Interrupted) => Interaction::CtrlC,
            Err(ReadlineError::Eof) => Interaction::CtrlD,
            Err(_err) => {
                Interaction::Err // TODO
            }
        };

        let response = handler.receive_interaction(interaction);
        match response {
            Response::Exit => break,
            Response::Continue => {}
        }
    }

    if let Some(history_file) = &history_file {
        rl.save_history(history_file)
            .map_err(PromptError::ReadLine)?;
    }

    Ok(())
}

/// Spawn `Interact` in a new thread.
pub fn spawn<H>(settings: Settings, handler: H) -> std::thread::JoinHandle<()>
where
    H: Handler + Send + Sync + 'static,
{
    thread::spawn(move || {
        let _ = direct(settings, handler);
    })
}
