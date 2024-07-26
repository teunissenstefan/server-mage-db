use std::fs;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use serde::Deserialize;
use std::process::Command;
use serde_json::Value;

fn clear_screen() {
    println!("\x1B[2J\x1B[1;1H");
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
    let environment_selections = &[
        "production",
        "staging",
    ];

    clear_screen();

    let environment_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an environment")
        .default(0)
        .items(&environment_selections[..])
        .interact()
        .unwrap();

    let mut environment_filename = "/Users/stefan-hypr/Tools/mage-db-sync-databases/staging.json";
    if environment_selections[environment_selection] == "production" {
        environment_filename = "/Users/stefan-hypr/Tools/mage-db-sync-databases/production.json";
    }

    let file = fs::File::open(environment_filename)
        .expect("Unable to open file as read-only!");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("File is invalid JSON!");
    let databases = json.get("databases")
        .expect("File should have a 'databases' key!");

    let mut stored_servers: Vec<Server> = vec![];
    let mut server_selections = vec![];
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

    let server_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a server")
        .default(0)
        .max_length(10)
        .items(&server_selections[..])
        .interact()
        .unwrap();

    let username = stored_servers[server_selection].username.to_owned();
    let server = stored_servers[server_selection].server.to_owned();
    let port = stored_servers[server_selection].port.to_owned();
    let username_server = format!("{username}@{server}");
    // println!("{}", username_server);
    Command::new("ssh")
        .arg(username_server)
        .arg("-p")
        .arg(port)
        .spawn().expect("Failed to execute ssh command")
        .wait().expect("Panic while running ssh command");
}