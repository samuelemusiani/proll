use clap::Parser;
use colored::Colorize;

mod cli;

fn main() {
    let cli = cli::Cli::parse();

    let res = match cli.command {
        Some(cli::Commands::List {
            package,
            raw,
            no_color,
        }) => cli::list(package, raw, no_color),
        Some(cli::Commands::Downgrade { package, version }) => cli::downgrade(package, version),
        None => panic!(),
    };

    match res {
        Ok(s) => print!("{s}"),
        Err(e) => {
            eprint!("{}: {e}", "Error".red());
            std::process::exit(1);
        }
    };
}
