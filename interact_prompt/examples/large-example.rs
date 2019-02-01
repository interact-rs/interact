extern crate interact;
extern crate structopt_derive;

use interact_prompt::{LocalRegistry, SendRegistry, Settings};

mod common;
use common::Rand;

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
    let seed = 42;
    let mut rng: rand::StdRng = rand::SeedableRng::seed_from_u64(seed);

    use common::{Basic, Complex, LocalRcLoop};

    SendRegistry::insert("complex", Box::new(Complex::new_random(&mut rng)));
    SendRegistry::insert("basic", Box::new(Basic::new_random(&mut rng)));
    LocalRegistry::insert("rc_loops", Box::new(LocalRcLoop::new_random(&mut rng)));

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
