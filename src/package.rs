use std::{
    fs::File,
    path::{Path, PathBuf},
    process,
};

use owo_colors::OwoColorize;
use shared::plugin_system::PluginManifest;

use crate::LoadMetadata;

pub fn handle_package_command(path: PathBuf) -> PluginManifest {
    let metadata = PluginManifest::read_from_dir(&path);

    println!(
        "Creating archive for plugin \"{}\" with version \"{}\"",
        metadata.name, metadata.version
    );
    validate_includes(&path, &metadata);
    create_archive(path, &metadata);
    println!("{}", "Done!".green().bold());
    metadata
}

/// Fail loudly *before* writing an archive when declared files are missing.
/// A zero-match `include_files` glob or an absent `bin` executable would
/// otherwise ship a valid-looking package that the backend can't start,
/// surfacing only as a confusing "Disabled" status after upload.
fn validate_includes(path: &Path, metadata: &PluginManifest) {
    let mut errors: Vec<String> = Vec::new();

    // Every include_files pattern must match at least one real file.
    for pattern in &metadata.include_files {
        let matched = glob::glob(pattern)
            .map(|paths| paths.flatten().any(|p| p.is_file()))
            .unwrap_or(false);
        if !matched {
            errors.push(format!("include_files pattern \"{pattern}\" matched no files"));
        }
    }

    // The bin program (first whitespace token, e.g. the executable in
    // "./bin/bun run index.js") must exist when it is a path, not a bare
    // command resolved from PATH.
    if let Some(bin) = &metadata.bin {
        let program = bin.split_whitespace().next().unwrap_or(bin);
        if program.starts_with('.') || program.contains('/') {
            let full = path.join(program.trim_start_matches("./"));
            if !full.is_file() {
                errors.push(format!(
                    "bin program \"{program}\" does not exist (looked for {})",
                    full.display()
                ));
            }
        }
    }

    // The declared hero image must exist so the registry has a file to extract
    // and host. Its format/size are only recommendations, so warn instead of
    // failing when they are off.
    if let Some(hero) = &metadata.hero_image {
        let full = path.join(hero);
        if !full.is_file() {
            errors.push(format!(
                "hero_image \"{hero}\" does not exist (looked for {})",
                full.display()
            ));
        } else {
            const RECOMMENDED_MAX_BYTES: u64 = 500 * 1024;
            let ext = full.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
            if !matches!(ext.as_deref(), Some("png" | "jpg" | "jpeg" | "webp" | "svg")) {
                eprintln!(
                    "{} hero_image \"{hero}\" has an unexpected format; recommended: PNG, JPG, WebP, SVG",
                    "warning:".yellow().bold()
                );
            }
            if let Ok(meta) = full.metadata() {
                if meta.len() > RECOMMENDED_MAX_BYTES {
                    eprintln!(
                        "{} hero_image \"{hero}\" is {} KB, over the recommended {} KB cap",
                        "warning:".yellow().bold(),
                        meta.len() / 1024,
                        RECOMMENDED_MAX_BYTES / 1024
                    );
                }
            }
        }
    }

    if !errors.is_empty() {
        eprintln!(
            "{}",
            "Refusing to package — the archive would be incomplete:"
                .red()
                .bold()
        );
        for e in &errors {
            eprintln!("  {} {e}", "✗".red());
        }
        eprintln!(
            "\nBuild the plugin first (e.g. `just build`) so every declared file exists, then re-run package."
        );
        process::exit(1);
    }
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

    // Ship the hero image at its declared relative path so the registry can
    // extract and host it. Dedupe against include_files matches so a hero that is
    // also globbed in doesn't produce a duplicate tar entry — authors therefore
    // don't need to list it in include_files.
    if let Some(hero) = &metadata.hero_image {
        let hero_path = path.join(hero);
        if hero_path.is_file() {
            let hero_canonical = hero_path.canonicalize().ok();
            let already_included = metadata.include_files.iter().any(|pattern| {
                glob::glob(pattern)
                    .map(|paths| paths.flatten().any(|p| p.canonicalize().ok() == hero_canonical))
                    .unwrap_or(false)
            });
            if !already_included {
                tar.append_path_with_name(&hero_path, hero).unwrap();
            }
        }
    }

    tar.finish().unwrap();
}
