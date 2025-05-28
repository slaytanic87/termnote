use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::{error::Error, fs::File};

use serde::{Deserialize, Serialize};

const CONFIG_FOLDER: &str = "/.termnote";
const FILE_PATH: &str = "/.termnote/db.json";

#[derive(Deserialize, Serialize, Clone)]
pub struct Topic {
    pub title: String,
    pub description: String,
    pub command: String,
}

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub topics: Vec<Topic>,
}

pub struct ObjectDB {
    pub library: Library,
}

impl ObjectDB {
    pub fn init() -> Self {
        let home_dir_rs = dirs::home_dir();

        let path_buf = match home_dir_rs {
            Some(path) => path,
            None => panic!("Could not find home directory"),
        };

        let home_dir = path_buf.display().to_string();
        let result: Result<Library, Box<dyn Error>> = {
            let file_rs = File::open(format!("{}/{}", home_dir, FILE_PATH));

            match file_rs {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let library = serde_json::from_reader(reader);
                    if library.is_err() {
                        panic!("Could not parse file cause: {}", library.err().unwrap());
                    }
                    Ok(library.unwrap())
                }
                Err(e) => Err(format!("Could not open file cause: {}", e).into()),
            }
        };

        if let Ok(rs) = result {
            return Self { library: rs };
        }
        let config_path: String = format!("{}/{}", home_dir, CONFIG_FOLDER);
        if !Path::new(&config_path).exists() {
            let rs = fs::create_dir(config_path);
            if let Err(error_msg) = rs {
                panic!(
                    "Could not create configuration directory in home folder cause: {}",
                    error_msg
                )
            }
        }

        let file_path: String = format!("{}/{}", home_dir, FILE_PATH);
        let lib = Library { topics: Vec::new() };
        if !Path::new(&file_path).exists() {
            let file_rs = File::create(format!("{}/{}", home_dir, FILE_PATH));
            if let Err(e) = file_rs {
                panic!("Could not create file cause: {}", e);
            }
            let file: File = file_rs.unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &lib).expect("Could not create empty database");
            writer.flush().expect("could not write file");
        }
        Self { library: lib }
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let home_dir_rs = dirs::home_dir();
        if home_dir_rs.is_none() {
            panic!("Could not find home directory");
        }
        let path_buf = home_dir_rs.unwrap();
        let home_dir = path_buf.display().to_string();

        let file = File::create(format!("{}/{}", home_dir, FILE_PATH))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.library)?;
        writer.flush()?;
        Ok(())
    }
}
