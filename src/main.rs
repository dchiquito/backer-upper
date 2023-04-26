use clap::Parser;

mod commands;
// mod context;

// #[cfg(test)]
// mod test;

fn main() {
    let cli = commands::Cli::parse();
    cli.run_command().unwrap_or_else(|x| {
        println!("{}", x);
    });
}
