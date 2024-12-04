use clap::Command;

mod sys_info;

fn cli() -> Command {
    Command::new("monitr")
        .about("A system resource monitor")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("run").about("Run the monitor"))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("run", _)) => {
            println!("Running monitor...");
            sys_info::print();
        }
        _ => unreachable!(),
    }
}
