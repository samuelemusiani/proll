use clap::Parser;

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    let res = match cli.command {
        Some(cli::Commands::List {
            package,
            raw,
            no_color,
        }) => cli::list(&package, raw, no_color),
        None => panic!(),
    };

    match res {
        Ok(s) => print!("{s}"),
        Err(e) => return Err(e),
    };

    Ok(())
}
