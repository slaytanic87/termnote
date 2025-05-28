use execute::{shell, Execute};
use std::process::Stdio;

use crate::adapter::Topic;
use crate::ObjectDB;

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

    pub fn add(&mut self, title: String, description: String, cmd: String) -> String {
        let topic_entry = Topic {
            title: title.clone(),
            description,
            command: cmd,
        };
        let topic: Option<&Topic> = self
            .database
            .library
            .topics
            .iter()
            .find(|tpc| tpc.title.to_lowercase() == title.to_lowercase());
        if topic.is_some() {
            return "Error: Topic already exists".to_string();
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

    pub fn remove_by_title(&mut self, title: String) -> String {
        let index: Option<usize> = self
            .database
            .library
            .topics
            .iter()
            .position(|topic| topic.title == title);
        if index.is_none() {
            return "Error: Could not find topic".to_string();
        }
        self.database.library.topics.remove(index.unwrap());
        if self.database.save().is_err() {
            return "Error: Could not update the database".to_string();
        }
        "Success: Removed topic".to_string()
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
