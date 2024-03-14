use std::process;

use inquire::{Password, Text};
use owo_colors::{colors::xterm, OwoColorize};

use crate::{
    config::{Config, ConfigDir},
    package::handle_package_command,
    store_api::{Client, ClientError},
    utils::SoftPanic,
};

pub fn handle_publish_command(
    user_config: &mut Config,
    user_config_dir: ConfigDir,
    remote: Option<String>,
    build: bool,
) {
    let metadata = handle_package_command(None, build); // package the current directory
    println!("Publishing plugin...");
    let email = user_config.user.clone().unwrap_or_else(|| {
        let email = Text::new("Email").prompt().expect("Email is required");
        user_config.user = Some(email.clone());
        user_config.save(&user_config_dir);
        email
    });
    println!();
    let client = Client::new(
        remote.unwrap_or("http://localhost:5000".to_string()),
        &email,
    );
    let client = if !client.is_logged_in() {
        let pw = Password::new("Password")
            .without_confirmation()
            .prompt()
            .expect("Password is required");
        println!();
        client.login(&email, Some(pw)).unwrap_or_else(|e| {
            println!("{} {}", "Could not login:".red(), e.to_string().red());
            process::exit(1);
        })
    } else {
        client.login(&email, None).unwrap_or_else(|e| {
            println!("{} {}", "Could not login:".red(), e.to_string().red());
            process::exit(1);
        })
    };
    match client.publish_plugin(&metadata) {
        Ok(_) => {}
        Err(ClientError::NotLoggedIn) => {
            println!("{}", "Publish failed,".red());
            println!("Retrying...");
            let client = client.logout().unwrap_or_else(|e| {
                println!(
                    "{} {}. Please try again manually",
                    "Could not retry:".red(),
                    e.to_string().red()
                );
                process::exit(1);
            });
            let pw = Password::new("Password")
                .without_confirmation()
                .prompt()
                .soft_expect("Password is required");
            println!();
            let client = client.login(&email, Some(pw)).unwrap_or_else(|e| {
                println!("{} {}", "Could not login:".red(), e.to_string().red());
                process::exit(1);
            });
            client.publish_plugin(&metadata).unwrap_or_else(|e| {
                println!(
                    "{} {}",
                    "Could not publish plugin:".red(),
                    e.to_string().red()
                );
                process::exit(1);
            });
        }
        Err(e) => {
            println!(
                "{} {}",
                "Could not publish plugin:".red(),
                e.to_string().red()
            );
            process::exit(1);
        }
    };
    println!("Plugin published successfully!");
    println!("{}", "–––––––––––––––––––––––––––".fg::<xterm::Gray>());
    println!("Plugin name: {}", metadata.name.green());
    println!("Plugin version: {}", metadata.version.yellow());
    println!("{}", "–––––––––––––––––––––––––––".fg::<xterm::Gray>());
}
