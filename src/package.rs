use std::{
    env,
    fs::File,
    path::PathBuf,
    process::{self, Command},
};

use ignore::Walk;
use owo_colors::OwoColorize;

use crate::PluginMetadata;

pub fn handle_package_command(path: Option<PathBuf>, build: bool) -> PluginMetadata {
    let path = path.unwrap_or_else(|| env::current_dir().expect("Could not get current directory"));
    let metadata_path = path.join("plugin.json");
    let metadata = if metadata_path.exists() {
        let file = File::open(metadata_path).unwrap();
        let metadata: PluginMetadata = serde_json::from_reader(file).unwrap();
        metadata
    } else {
        println!("Plugin metadata not found");
        process::exit(1);
    };
    println!(
        "Creating archive for plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
    if build {
        Command::new("bun").args(["run", "build"]).output().unwrap();
    }
    create_archive(path, &metadata);
    println!("{}", "Done!".green().bold());
    metadata
}

fn create_archive(path: PathBuf, metadata: &PluginMetadata) {
    let archive = File::create(metadata.archive_name()).unwrap();
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
    tar.finish().unwrap();
}
