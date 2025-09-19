use clap::{arg, ArgMatches, Command};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::Stdout;
use termnote::{display_text, run_cmd, CRUDProcessor, MenuEvent, TerminalUI};

fn cmd() -> Command {
    Command::new("tn")
        .about("A terminal CLI tool to note the commands and urls")
        .author("Slaytanic87")
        .subcommand(
            Command::new("add")
                .about("Add a new command to the list")
                .arg(arg!(-t --title <TITLE>))
                .arg(arg!(-d --description <DESCRIPTION>))
                .arg(arg!(-c --command <COMMAND>))
                .arg(arg!(-k --category <CATEGORY>)),
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
                .arg(arg!(-i --index <INDEX> "(mandatory) Index of the command to remove")),
        )
        .subcommand(Command::new("search")
                    .about("Search commands by title or description")
                    .arg(arg!(-q --query <QUERY> "(mandatory) Query string to search for in titles or descriptions"))
        )
        .subcommand(Command::new("list").about("List all noted commands"))
        .subcommand(Command::new("url")
                    .about("Link notes")
                    .subcommand(
                        Command::new("list").about("List all noted links")
                    )
                    .subcommand(Command::new("add")
                                .about("Add a new link to the list")
                                .arg(arg!(-t --title <TITLE>))
                                .arg(arg!(-u --url <URL>))
                    )
                    .subcommand(Command::new("update").about("Update a noted link")
                                .arg(arg!(-i --index <INDEX> "(mandatory) Index of the link to update"))
                                .arg(arg!(-t --title <TITLE>))
                                .arg(arg!(-u --url <URL>))
                    )
                    .subcommand(Command::new("remove").about("Remove a noted link by title or index")
                                .arg(arg!(-i --index <INDEX> "(mandatory) Index of the link to remove"))
                    )
                    .subcommand(Command::new("search").about("Search links by title")
                                .arg(arg!(-q --query <QUERY> "(mandatory) Query string to search for in link titles"))
                    )
        )
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
    let mut terminal_url_ui = termnote::TerminalUrlUI::new(processor.database.library.links.clone());
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
            let category: &str = sub_matches
                .get_one::<String>("category")
                .expect("Category is required");
            processor.add(
                title.to_string(),
                description.unwrap_or("").to_string(),
                command.to_string(),
                category.to_string(),
            )
        }
        Some(("remove", sub_matches)) => {
            let index_str: Option<String> = sub_matches.get_one::<String>("index").cloned();
            if let Some(idx_str) = index_str {
                if let Ok(idx) = idx_str.parse() {
                    processor.remove_by_index(idx)
                } else {
                    "Invalid index number".to_string()
                }
            } else {
                "Error: Must provide an index".to_string()
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
        Some(("search", sub_matches)) => {
            let query: &String = sub_matches
                .get_one::<String>("query")
                .expect("Query is required");
            let results = processor.search_by_title_or_description(query);
            if results.is_empty() {
                "No commands found matching the query".to_string()
            } else {
                termnote::deserialize_topics(&results)
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
        Some(("url", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("list", _)) => {
                    terminal_url_ui.menu_loop(&mut terminal)?;
                    let selected_url:String = terminal_url_ui.selected_url;
                    if !selected_url.is_empty() {
                        selected_url
                    } else {
                        "".to_string()
                    }
                }
                Some(("add", url_matches)) => {
                    let title = url_matches
                        .get_one::<String>("title")
                        .expect("Title is required");
                    let url = url_matches
                        .get_one::<String>("url")
                        .expect("URL is required");
                    processor.add_url(title.to_string(), url.to_string())
                }
                Some(("update", url_matches)) => {
                    let index_str: &String = url_matches
                        .get_one::<String>("index")
                        .expect("Index is required");
                    let title = url_matches.get_one::<String>("title");
                    let url = url_matches.get_one::<String>("url");
                    if let Ok(idx) = index_str.parse() {
                        processor.update_url(idx, title, url)
                    } else {
                        "Invalid index number".to_string()
                    }
                }
                Some(("remove", url_matches)) => {
                    let index_str: Option<String> = url_matches.get_one::<String>("index").cloned();
                    if let Some(idx_str) = index_str {
                        if let Ok(idx) = idx_str.parse() {
                            processor.remove_url_by_index(idx)
                        } else {
                            "Invalid index number".to_string()
                        }
                    } else {
                        "Error: Must provide an index".to_string()
                    }
                }
                Some(("search", url_matches)) => {
                    let query: &String = url_matches
                        .get_one::<String>("query")
                        .expect("Query is required");
                    let results = processor.search_links_by_title(query);
                    if results.is_empty() {
                        "No links found matching the query".to_string()
                    } else {
                        termnote::deserialize_links(&results)
                    }
                }
                _ => "Missing subcommand for url".to_string(),
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
