use home::home_dir;
use rusqlite::Connection;

use clap::Command;

mod sys_info;

fn cli() -> Command {
    Command::new("monitr")
        .about("A system resource monitor")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("run").about("Run the monitor"))
        .subcommand(Command::new("view").about("View the monitor history"))
}

fn main() {
    let matches = cli().get_matches();

    let path = home_dir().unwrap_or_default().as_path().join(".monitr");
    let conn = Connection::open(path.join("system_info.db")).unwrap();

    sys_info::create_tables(&conn);

    match matches.subcommand() {
        Some(("run", _)) => {
            println!("Running monitor...");

            loop {
                sys_info::log_system_info(&conn);

                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
        Some(("view", _)) => {
            println!("Viewing monitor...");

            sys_info::print_system_info(&conn);
        }
        _ => unreachable!(),
    }
}
