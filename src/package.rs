use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use ignore::Walk;
use owo_colors::OwoColorize;
use sha2::{Digest, Sha256};

use crate::{utils::SoftPanic, PluginMetadata, PluginMetadataType};

pub fn handle_package_command(path: PathBuf) -> PluginMetadata {
    let metadata = PluginMetadata::read_from_dir(&path);

    println!(
        "Building plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
    build_plugin(&path, &metadata);
    println!(
        "Creating archive for plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
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
    if path.join("build").exists() {
        fs::remove_dir_all(path.join("build")).unwrap();
    }
    let build_command = &metadata.build;
    fs::create_dir_all(path.join("build")).unwrap();
    if let Some(build_command) = build_command {
        let mut build = build_command.split_whitespace();
        let mut running = Command::new(build.next().unwrap())
            .args(build)
            .current_dir(path)
            .spawn()
            .soft_expect("Could not run build command");
        running.wait().unwrap();
    }
    let plugin_json = path.join("plugin.json");
    let readme_md = path.join("readme.md");
    let build = path.join(".plugin-build");
    std::fs::copy(plugin_json, build.join("plugin.json")).soft_expect("plugin.json file not found");
    std::fs::copy(readme_md, build.join("readme.md")).soft_expect("readme.md file not found");

    // copy all files specified in "files" to the temp output dir
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

    let built_files = fs::read_dir(&build).unwrap();
    match &metadata.ty {
        PluginMetadataType::Module { main } => {
            let mut new_main = None;
            for file in built_files {
                let path = file.unwrap().path();
                if path.file_name().unwrap() == "plugin.json"
                    || path.file_name().unwrap() == "readme.md"
                {
                    continue;
                }

                let mut hasher = Sha256::new();
                let mut file = File::open(&path).unwrap();
                std::io::copy(&mut file, &mut hasher).unwrap();
                let result = hasher.finalize();
                let new_file_name = format!(
                    "{}-{:x}.{}",
                    path.file_stem().unwrap().to_str().unwrap(),
                    result,
                    path.extension().unwrap().to_str().unwrap()
                );
                if path.file_name().unwrap().to_str().unwrap() == main {
                    new_main = Some(new_file_name.clone());
                }
                fs::rename(&path, path.parent().unwrap().join(&new_file_name)).unwrap();
            }

            let new_metadata = PluginMetadata {
                ty: PluginMetadataType::Module {
                    main: new_main.unwrap(),
                },
                ..metadata.clone()
            };
            fs::write(
                build.join("plugin.json"),
                serde_json::to_string_pretty(&new_metadata).unwrap(),
            )
            .unwrap();
        }
        PluginMetadataType::Service { bin, .. } => {
            std::fs::copy(bin, build.join(bin))
                .soft_expect("Couldn't copy plugin binary to output directory");
        }
    }
}
