use std::{fs::File, path::PathBuf};

use ignore::Walk;
use owo_colors::OwoColorize;
use shared::plugin_system::PluginManifest;

use crate::LoadMetadata;

pub fn handle_package_command(path: PathBuf) -> PluginManifest {
    let metadata = PluginManifest::read_from_dir(&path);

    println!(
        "Creating archive for plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
    create_archive(path, &metadata);
    println!("{}", "Done!".green().bold());
    metadata
}

fn create_archive(path: PathBuf, metadata: &PluginManifest) {
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
