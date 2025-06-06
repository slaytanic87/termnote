use clap::{arg, ArgMatches, Command};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::Stdout;
use termnote::{display_text, run_cmd, CRUDProcessor, MenuEvent, TerminalUI};

fn cmd() -> Command {
    Command::new("termnote")
        .about("A terminal CLI tool to note the commands you run")
        .author("Slaytanic87")
        .subcommand(
            Command::new("add")
                .about("Add a new command to the list")
                .arg(arg!(-t --title <TITLE>))
                .arg(arg!(-d --description <DESCRIPTION>))
                .arg(arg!(-c --command <COMMAND>)),
        )
        .subcommand(
            Command::new("update")
                .about("Update a noted command")
                .arg(arg!(-i --index <INDEX> "(mandatory) Index of the command to update"))
                .arg(arg!(-t --title <TITLE>))
                .arg(arg!(-d --description <DESCRIPTION>))
                .arg(arg!(-c --command <COMMAND>)),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a noted command by title or index")
                .arg(arg!(-t --title <TITLE>))
                .arg(arg!(-i --index <INDEX> "(mandatory) Index of the command to remove")),
        )
        .subcommand(Command::new("list").about("List all noted commands"))
}

fn restore_terminal() -> Result<(), Box<dyn Error>> {
    ratatui::restore();
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches: ArgMatches = cmd().get_matches();
    let mut processor = CRUDProcessor::init();
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = ratatui::init();
    let mut terminal_ui = TerminalUI::new(processor.database.library.topics.clone());
    let message: String = match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let title = sub_matches
                .get_one::<String>("title")
                .expect("Title is required");
            let description = sub_matches
                .get_one::<String>("description")
                .map(|s| s.as_str());
            let command: &str = sub_matches
                .get_one::<String>("command")
                .expect("Command is required");
            processor.add(
                title.to_string(),
                description.unwrap_or("").to_string(),
                command.to_string(),
            )
        }
        Some(("remove", sub_matches)) => {
            let title: Option<String> = sub_matches.get_one::<String>("title").cloned();
            let index_str: Option<String> = sub_matches.get_one::<String>("index").cloned();
            if let Some(idx_str) = index_str {
                if let Ok(idx) = idx_str.parse() {
                    processor.remove_by_index(idx)
                } else {
                    "Invalid index number".to_string()
                }
            } else if let Some(tte) = title {
                processor.remove_by_title(tte)
            } else {
                "Error: Must provide either title or index".to_string()
            }
        }
        Some(("update", sub_matches)) => {
            let index_str: &String = sub_matches.get_one::<String>("index").expect("Index is required");
            let title = sub_matches.get_one::<String>("title");
            let description = sub_matches.get_one::<String>("description");
            let command = sub_matches.get_one::<String>("command");
            if let Ok(idx) = index_str.parse() {
                processor.update(idx, title, description, command)
            } else {
                "Invalid index number".to_string()
            }
        }
        Some(("list", _)) => {
            terminal_ui.menu_loop(&mut terminal)?;
            match terminal_ui.event {
                MenuEvent::Execute => run_cmd(&terminal_ui.selected_cmd.to_owned()),
                MenuEvent::Display => terminal_ui.selected_cmd,
                _ => "".to_string(),
            }
        }
        _ => "Missing subcommand!".to_string(),
    };
    restore_terminal()?;
    if !message.is_empty() {
        display_text(&message);
    }
    Ok(())
}
