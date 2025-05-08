use std::env;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;

use clap::Parser;
use config::Config;
use config::ConfigDir;
use fs_extra::dir::CopyOptions;
use ignore::overrides::OverrideBuilder;
use ignore::Walk;
use ignore::WalkBuilder;
use inquire::Select;
use inquire::{Confirm, Text};
use owo_colors::colors::*;
use owo_colors::OwoColorize;
use package::handle_package_command;
use publish::handle_publish_command;
use serde::Serialize;
use shared::plugin_system::PluginManifest;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::command::Cli;

pub mod command;
pub mod config;
pub mod package;
pub mod publish;
pub mod store_api;
pub mod utils;

const TEMPLATE_REPO: &str = "https://github.com/Triple-A-Software/plugin-template";

#[derive(Serialize, Eq, PartialEq, EnumIter)]
#[serde(rename_all = "kebab-case")]
enum LanguageOption {
    Rust,
    Typescript,
    Python,
    Php,
    Go,
}

impl Display for LanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageOption::Rust => f.write_str("Rust"),
            LanguageOption::Typescript => f.write_str("Typescript"),
            LanguageOption::Python => f.write_str("Python"),
            LanguageOption::Php => f.write_str("PHP"),
            LanguageOption::Go => f.write_str("Go"),
        }
    }
}

impl LanguageOption {
    fn to_dir_name(&self) -> &'static str {
        match self {
            LanguageOption::Rust => "rust",
            LanguageOption::Typescript => "typescript",
            LanguageOption::Python => "python",
            LanguageOption::Php => "php",
            LanguageOption::Go => "go",
        }
    }
}

fn main() {
    let command = Cli::parse();

    let user_config_dir = ConfigDir::new();
    let mut user_config = Config::from(&user_config_dir);

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
            let (template, language) = if let Some(template) = template {
                (template, None)
            } else {
                let language = Select::new(
                    "Language",
                    vec![
                        LanguageOption::Rust,
                        LanguageOption::Typescript,
                        LanguageOption::Go,
                        LanguageOption::Python,
                        LanguageOption::Php,
                    ],
                )
                .prompt()
                .expect("You need to select a programming language");

                (TEMPLATE_REPO.to_string(), Some(language))
            };

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
                        println!(
                            "{} {}",
                            "Could not clone template repository:".red(),
                            template
                        );
                        process::exit(1);
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
            let globals = liquid::object!({ "plugin_name": &name, "project_name": &name, "language": language });

            // the language is only set when using the default template repo
            // remove all files not for this language and move the files from the languge specific
            // folder into the root
            if let Some(ref language) = language {
                for option in LanguageOption::iter() {
                    if *language != option {
                        std::fs::remove_dir_all(option.to_dir_name()).unwrap();
                    }
                }
                fs_extra::dir::move_dir(language.to_dir_name(), ".", &CopyOptions::new()).unwrap();
            }
            for entry in Walk::new(&name).filter_map(Result::ok) {
                if entry.file_type().unwrap().is_dir() {
                    continue;
                }
                let contents = std::fs::read_to_string(entry.path()).unwrap();
                let template_parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
                let template = template_parser.parse(&contents).unwrap();
                let output = template.render(&globals).unwrap();
                let generated_file_name = if entry.path().ends_with(".liquid") {
                    entry.path().with_extension("")
                } else {
                    entry.path().to_path_buf()
                };
                std::fs::write(generated_file_name, output).unwrap();
            }

            println!("{}", "Done!".green().bold());
            if init_git {
                println!("Initializing git repository...");
                let _ = Command::new("git")
                    .arg("init")
                    .arg(&name)
                    .status()
                    .unwrap_or_else(|_| {
                        println!("{} {}", "Could not initialize git repository:".red(), name);
                        process::exit(1);
                    });
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
            match language {
                Some(LanguageOption::Typescript) => {
                    println!(
                        "{} {} {}",
                        "$".fg::<xterm::Gray>(),
                        "bun".italic().yellow(),
                        "install".red()
                    );
                }
                Some(_) => {}
                None => {}
            }
            println!("{}", "–––––––––––––––––––––––––––".fg::<xterm::Gray>());
        }
        Cli::Package { path } => {
            let path = path
                .unwrap_or_else(|| env::current_dir().expect("Could not get current directory"));
            handle_package_command(path);
        }
        Cli::Publish { remote } => {
            let path = env::current_dir().expect("Could not get current directory");
            handle_publish_command(path, &mut user_config, user_config_dir, remote);
        }
    }
}

pub trait LoadMetadata {
    fn archive_name(&self) -> String;
    fn read_from_dir(path: &Path) -> Self;
}

impl LoadMetadata for PluginManifest {
    fn archive_name(&self) -> String {
        format!(
            "{name}-{version}.tar.gz",
            name = self.name,
            version = self.version
        )
    }

    fn read_from_dir(path: &Path) -> PluginManifest {
        let metadata_path = path.join("plugin.json");

        if metadata_path.exists() {
            let file = File::open(metadata_path).unwrap();
            let metadata: PluginManifest = serde_json::from_reader(file).unwrap();
            metadata
        } else {
            println!("Plugin metadata not found");
            process::exit(1);
        }
    }
}
