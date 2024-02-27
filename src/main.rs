use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use ignore::overrides::OverrideBuilder;
use ignore::Walk;
use ignore::WalkBuilder;
use inquire::{Confirm, Text};
use owo_colors::colors::*;
use owo_colors::OwoColorize;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;

use crate::command::Cli;

pub mod command;

const TEMPLATE_REPO: &str = "https://github.com/Triple-A-Software/plugin-template";

fn main() {
    let command = Cli::parse();

    match command {
        Cli::Create {
            name,
            template,
            git_template,
        } => {
            let name = name.unwrap_or_else(|| {
                Text::new("Plugin name")
                    .prompt()
                    .expect("Plugin name is required")
            });
            let init_git = Confirm::new("Do you want to initialize a git repository?")
                .with_default(true)
                .prompt()
                .unwrap_or(true);

            let git_template = git_template.unwrap_or(true);
            let template = template.unwrap_or(TEMPLATE_REPO.to_string());

            println!();
            println!("Copying code for plugin \"{}\"", name.bold());

            if git_template {
                // spawn process to clone template repo into project-name directory
                let _ = Command::new("git")
                    .arg("clone")
                    .arg(&template)
                    .arg(&name)
                    .arg("--depth")
                    .arg("1")
                    .status()
                    .unwrap_or_else(|_| {
                        panic!("Could not clone template repository: {}", template)
                    });
                std::fs::remove_dir_all(PathBuf::from(&name).join(".git")).unwrap();
            } else {
                // copy template into project-name directory
                let template_path = PathBuf::from(template);
                let project_path = PathBuf::from(&name);
                let walker = &mut WalkBuilder::new(&template_path)
                    .overrides(
                        // it should copy the .gitignore file as well, as normally hidden files are
                        // excluded
                        OverrideBuilder::new(&template_path)
                            .add("!.gitignore")
                            .unwrap()
                            .build()
                            .unwrap(),
                    )
                    .build();
                for entry in walker {
                    match entry {
                        Ok(entry) => {
                            let dest = project_path
                                .join(entry.path().strip_prefix(&template_path).unwrap());
                            if entry.file_type().unwrap().is_dir() {
                                std::fs::create_dir_all(&dest).unwrap();
                            } else {
                                std::fs::copy(entry.path(), &dest).unwrap();
                            }
                        }
                        Err(err) => println!("Error: {}", err),
                    }
                }
            }
            // substitute project-name into files
            let globals = liquid::object!({ "plugin_name": &name, "project_name": &name });
            for entry in Walk::new(&name).filter_map(Result::ok) {
                if entry.file_type().unwrap().is_dir() {
                    continue;
                }
                let contents = std::fs::read_to_string(entry.path()).unwrap();
                let template_parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
                let template = template_parser.parse(&contents).unwrap();
                let output = template.render(&globals).unwrap();
                std::fs::write(entry.path(), output).unwrap();
            }

            println!("{}", "Done!".green().bold());
            if init_git {
                println!("Initializing git repository...");
                let _ = Command::new("git")
                    .arg("init")
                    .arg(&name)
                    .status()
                    .unwrap_or_else(|_| panic!("Could not initialize git repository: {}", name));
                println!("Done!");
            }
            println!("{}", "–––––––––––––––––––––––––––".fg::<xterm::Gray>());
            println!("{}", "Next steps:".bold());
            println!(
                "{} {} {}",
                "$".fg::<xterm::Gray>(),
                "cd".italic().yellow(),
                name
            );
            println!(
                "{} {} {}",
                "$".fg::<xterm::Gray>(),
                "bun".italic().yellow(),
                "install".red()
            );
            println!("{}", "–––––––––––––––––––––––––––".fg::<xterm::Gray>());
        }
        Cli::Package { path } => {
            let path = path
                .unwrap_or_else(|| env::current_dir().expect("Could not get current directory"));
            let metadata_path = path.join("plugin.json");
            let metadata = if metadata_path.exists() {
                let file = File::open(metadata_path).unwrap();
                let metadata: PluginMetadata = serde_json::from_reader(file).unwrap();
                metadata
            } else {
                panic!("Plugin metadata not found");
            };
            println!(
                "Creating archive for plugin \"{}\" with version \"{}\"",
                metadata.name, metadata.version
            );
            create_archive(path, metadata.name, metadata.version);
            println!("{}", "Done!".green().bold());
        }
        Cli::Publish => {
            todo!("Not yet implemented")
        }
    }
}

fn create_archive(path: PathBuf, name: String, version: Version) {
    let archive = File::create(format!("{name}-{version}.tar.gz")).unwrap();
    let enc = flate2::write::GzEncoder::new(archive, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    for entry in Walk::new(&path) {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    continue;
                }
                tar.append_path_with_name(entry_path, entry_path.strip_prefix(&path).unwrap())
                    .unwrap();
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PluginMetadata {
    name: String,
    version: Version,
}
