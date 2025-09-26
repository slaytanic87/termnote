use execute::{shell, Execute};
use std::process::Stdio;
use colored::Colorize;

use crate::{Link, Topic, ObjectDB};

pub struct CRUDProcessor {
    pub database: ObjectDB,
}

impl CRUDProcessor {
    pub fn init() -> Self {
        let object_db = ObjectDB::init();
        Self {
            database: object_db,
        }
    }

    pub fn add_url(&mut self, title: String, url: String) -> String {
        let link_entry = Link {
            title: title.clone(),
            url,
        };
        let link: Option<&Link> = self
            .database
            .library
            .links
            .iter()
            .find(|link| link.title.to_lowercase() == title.to_lowercase());
        if link.is_some() {
            return "Error: Link with that title already exists".to_string();
        }
        self.database.library.links.push(link_entry);
        if let Err(e) = self.database.save() {
            return format!("Error: Could not save to database cause: {}", e);
        }
        "Success: Added link".to_string()
    }

    pub fn add(&mut self, title: String, description: String, cmd: String, category: String) -> String {
        let topic_entry = Topic {
            title: title.clone(),
            description,
            command: cmd,
            category,
        };
        let topic: Option<&Topic> = self
            .database
            .library
            .topics
            .iter()
            .find(|tpc| tpc.title.to_lowercase() == title.to_lowercase());
        if topic.is_some() {
            return "Error: Command topic with that title already exists".to_string();
        }
        self.database.library.topics.push(topic_entry);
        if let Err(e) = self.database.save() {
            return format!("Error: Could not save to database cause: {}", e);
        }
        "Success: Added topic".to_string()
    }

    pub fn update(
        &mut self,
        index: usize,
        title: Option<&String>,
        description: Option<&String>,
        cmd: Option<&String>,
    ) -> String {
        if index >= self.database.library.topics.len() {
            return "Error: Could not find topic".to_string();
        }

        if title.is_none() && description.is_none() && cmd.is_none() {
            return "Error: No fields to update".to_string();
        }

        let topic = &mut self.database.library.topics[index];
        if let Some(new_title) = title {
            topic.title = new_title.to_string();
        }
        if let Some(new_description) = description {
            topic.description = new_description.to_string();
        }
        if let Some(new_command) = cmd {
            topic.command = new_command.to_string();
        }
        if self.database.save().is_err() {
            return "Error: Could not update the database".to_string();
        }
        "Success: Updated topic".to_string()
    }

    pub fn update_url(
        &mut self,
        index: usize,
        title: Option<&String>,
        url: Option<&String>,
    ) -> String {
        if index >= self.database.library.links.len() {
            return "Error: Could not find link".to_string();
        }

        if title.is_none() && url.is_none() {
            return "Error: No fields to update".to_string();
        }

        let link = &mut self.database.library.links[index];
        if let Some(new_title) = title {
            link.title = new_title.to_string();
        }
        if let Some(new_url) = url {
            link.url = new_url.to_string();
        }
        if self.database.save().is_err() {
            return "Error: Could not update the database".to_string();
        }
        "Success: Updated link".to_string()
    }

    pub fn remove_by_index(&mut self, index: usize) -> String {
        if index >= self.database.library.topics.len() {
            return "Error: Could not find topic".to_string();
        }
        self.database.library.topics.remove(index);
        if self.database.save().is_err() {
            return "Error: Could not update the database".to_string();
        }
        "Success: Removed topic".to_string()
    }

    pub fn search_by_title_or_description(&self, query: &str) -> Vec<&Topic> {
        self.database
            .library
            .topics
            .iter()
            .filter(|topic| topic.title.to_lowercase().contains(&query.to_lowercase()) || topic.description.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn remove_url_by_index(&mut self, index: usize) -> String {
        if index >= self.database.library.links.len() {
            return "Error: Could not find link".to_string();
        }
        self.database.library.links.remove(index);
        if self.database.save().is_err() {
            return "Error: Could not update the database".to_string();
        }
        "Success: Removed link".to_string()
    }

    pub fn search_links_by_title(&self, query: &str) -> Vec<&Link> {
        self.database
            .library
            .links
            .iter()
            .filter(|link| link.title.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }
}

pub fn run_cmd(cmd_str: &str) -> String {
    let mut command = shell(cmd_str);
    command.stdout(Stdio::piped());
    let output_rs = command.execute_output();
    if let Err(e) = output_rs {
        return format!("{}", e);
    }
    String::from_utf8(output_rs.unwrap().stdout).unwrap()
}

pub fn deserialize_links(links: &Vec<&Link>) -> String {
    let mut links_output: String = "".to_string();
    for (index, link) in links.iter().enumerate() {
        links_output.push_str(format!("{}: {} - {} \n", link.title, link.url.yellow()).as_str());
    });
    links_output
}

pub fn deserialize_topics(topics: &Vec<&Topic>) -> String {
    let mut topics_output: String = "".to_string();
    for (index, topic) in topics.iter().enumerate() {
        topics_output.push_str(
            format!(
                "{}: {} - {} \n",
                index, topic.title, topic.command.bright_green()
            )
            .as_str(),
        );
    };
    topics_output
}