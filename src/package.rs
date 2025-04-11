use std::{fs::File, path::PathBuf};

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
    for pattern in &metadata.include_files {
        for entry in glob::glob(pattern).unwrap().flatten() {
            let entry = entry.canonicalize().unwrap();
            if entry.is_dir() {
                continue;
            }
            tar.append_path_with_name(&entry, entry.strip_prefix(&path).unwrap())
                .unwrap();
        }
    }
    tar.append_path_with_name(path.join("plugin.json"), "plugin.json")
        .unwrap();
    let readme_path = path.join("readme.md");
    if readme_path.exists() {
        tar.append_path_with_name(readme_path, "readme.md").unwrap();
    }
    tar.finish().unwrap();
}
