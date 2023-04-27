use clap::Parser;

fn main() {
    let cli = backer_upper::commands::Cli::parse();
    cli.run_command().unwrap_or_else(|x| {
        println!("{}", x);
    });
}
