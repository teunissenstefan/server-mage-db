use std::fs;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use std::ops::Deref;
use std::path::Path;
use std::process::Command;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use serde::Deserialize;
use serde_json::Value;
use toml::Table;
use walkdir::{DirEntry, IntoIter, WalkDir};

const CONFIG_PATH: &'static str = "~/.config/server/config.toml";
fn first_time_setup() {
    println!("Config file {} not found!", CONFIG_PATH);

    fs::create_dir_all(shellexpand::tilde("~/.config/server/").to_string()).expect("Couldn't create directories");

    let mut file = OpenOptions::new().write(true).create(true).open(shellexpand::tilde(CONFIG_PATH).to_string()).unwrap();
    if let Err(e) = writeln!(file, "{}", "servers_dir = \"~/Tools/mage-db-sync-databases/\"") {
        eprintln!("Couldn't write to file: {}", e);
    }
    println!("Created an example. Please update the file and run this command again.");
}

#[derive(Deserialize, Debug)]
struct Server {
    username: String,
    server: String,
    port: String,
}

impl Server {
    pub fn new(
        username: String,
        server: String,
        port: String,
    ) -> Server {
        Server {
            username,
            server,
            port,
        }
    }
}


fn main() {
    if !Path::new(&shellexpand::tilde(CONFIG_PATH).to_string()).exists() {
        first_time_setup();
        return;
    }

    let config_values: Table = toml::from_str(&*fs::read_to_string(shellexpand::tilde(CONFIG_PATH).to_string()).unwrap()).unwrap();
    let binding: String = shellexpand::tilde(config_values["servers_dir"].as_str().unwrap()).to_string();
    let root_environment_dir: &str = binding.as_str();

    let walker: IntoIter = WalkDir::new(root_environment_dir).into_iter();
    let mut environment_selections: Vec<String> = vec![];
    for entry in walker {
        let dir_entry: DirEntry = entry.unwrap().to_owned();
        if !dir_entry.path().to_str().unwrap().ends_with(".json") { continue; }

        environment_selections.push(Path::new(dir_entry.path()).file_stem().unwrap().to_str().unwrap().to_owned());
    }

    //Clear screen and put cursor at the beginning
    println!("\x1B[2J\x1B[1;1H");

    let environment_selection: usize = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an environment")
        .default(0)
        .items(&environment_selections[..])
        .interact()
        .unwrap();

    let selected_env: String = environment_selections[environment_selection].to_owned();
    let environment_filename: String = format!("{root_environment_dir}{selected_env}.json");

    let file: File = File::open(environment_filename)
        .expect("Unable to open environment file as read-only!");
    let json: Value = serde_json::from_reader(file)
        .expect("File is invalid JSON!");
    let databases = json.get("databases")
        .expect("File should have a 'databases' key!");

    let mut stored_servers: Vec<Server> = vec![];
    let mut server_selections: Vec<String> = vec![];
    for database in databases.as_object().unwrap() {
        stored_servers.push(
            Server::new(
                database.1.get("username").unwrap().as_str().unwrap().to_string(),
                database.1.get("server").unwrap().as_str().unwrap().to_string(),
                database.1.get("port").unwrap_or(&Value::from("22")).as_str().unwrap_or("22").to_string(),
            )
        );
        server_selections.push(database.0.to_owned());
    }

    let server_selection: usize = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a server")
        .default(0)
        .max_length(10)
        .items(&server_selections[..])
        .interact()
        .unwrap();

    let username: String = stored_servers[server_selection].username.to_owned();
    let server: String = stored_servers[server_selection].server.to_owned();
    let port: String = stored_servers[server_selection].port.to_owned();
    let username_server: String = format!("{username}@{server}");
    // println!("{}", username_server);
    Command::new("ssh")
        .arg(username_server)
        .arg("-p")
        .arg(port)
        .spawn().expect("Failed to execute ssh command")
        .wait().expect("Panic while running ssh command");
}