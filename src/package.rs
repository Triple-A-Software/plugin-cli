use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{self, Command},
};

use ignore::Walk;
use owo_colors::OwoColorize;

use crate::{utils::SoftPanic, PluginMetadata};

pub fn handle_package_command(path: PathBuf) -> PluginMetadata {
    let metadata = PluginMetadata::read_from_dir(&path);

    println!(
        "Creating archive for plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
    build_plugin(&path, &metadata);
    create_archive(path.join("build"), &metadata);
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

fn build_plugin(path: &Path, metadata: &PluginMetadata) {
    let build_command = &metadata.build;
    fs::create_dir_all(path.join("build")).unwrap();
    if let Some(build_command) = build_command {
        let mut build = build_command.split_whitespace();
        Command::new(build.next().unwrap())
            .args(build)
            .current_dir(path)
            .output()
            .unwrap_or_else(|_| {
                println!("Could not build plugin");
                process::exit(1);
            });
    } else {
        Command::new("bun")
            .args(["build", ".", "--outdir", "./build"])
            .current_dir(path)
            .output()
            .unwrap_or_else(|_| {
                println!("Could not build plugin");
                process::exit(1);
            });
    };
    let plugin_json = path.join("plugin.json");
    let readme_md = path.join("readme.md");
    let build = path.join("build");
    std::fs::copy(plugin_json, build.join("plugin.json")).unwrap();
    std::fs::copy(readme_md, build.join("readme.md")).unwrap();

    if let Some(files) = &metadata.files {
        for file in files {
            for entry in glob::glob(file).soft_expect("Invalid glob pattern") {
                match entry {
                    Ok(path) => {
                        std::fs::copy(path, build.join(file)).unwrap();
                    }
                    Err(err) => println!("Error: {}", err.red()),
                }
            }
        }
    }
}
