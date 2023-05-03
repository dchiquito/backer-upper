use clap::Parser;
use log::error;

fn main() {
    env_logger::init();
    let cli = backer_upper::commands::Cli::parse();
    cli.run_command().unwrap_or_else(|x| {
        error!("{}", x);
    });
}
