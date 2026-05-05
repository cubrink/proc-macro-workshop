#[test]
fn test_snapshots() {
    // Setup
    remove_files("tests/ui/pass/*.expanded.rs");
    macrotest::expand("tests/ui/pass/*.rs");

    // Test
    insta::glob!("ui/pass/*.expanded.rs", |path| {
        let settings = build_snapshot_settings(path);
        settings.bind(|| test_snapshot(path));
    });

    // Tear down
    remove_files("tests/ui/pass/*.expanded.rs");
}

fn remove_files(glob_path: &str) {
    println!("Calling remove_files");
    let glob_matches = match glob::glob(glob_path) {
        Ok(paths) => {
            if paths.count() == 0 {
                println!("No files found to remove");
                return;
            } else {
                glob::glob(glob_path).unwrap()
            }
        }
        Err(e) => {
            println!("Error while globbing: {e}");
            return;
        }
    };

    println!(
        "Removing files {} [found {}]",
        glob_path,
        glob::glob(glob_path).unwrap().count()
    );
    for path in glob_matches {
        let path = path.unwrap();
        match std::fs::remove_file(path.as_os_str()) {
            Ok(_) => println!("Removed {}", path.display()),
            Err(e) => println!("Error deleting {}: {}", path.display(), e),
        }
    }
}

fn build_snapshot_settings(path: &std::path::Path) -> insta::Settings {
    let filename = match path.file_name() {
        Some(filename) => filename,
        None => panic!("Failed to get filename from {}", path.display()),
    };
    let description = format!("Source file: {}", filename.to_string_lossy());
    let mut settings = insta::Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_description(description);
    settings.set_omit_expression(true);
    settings.set_snapshot_suffix(filename.to_string_lossy());
    settings
}

fn test_snapshot(path: &std::path::Path) {
    // Snapshot testing cons
    // Convert a string representation to a syn::File
    // Pretty print the resulting syn::File
    let content = std::fs::read_to_string(path).unwrap();
    let file: syn::File = syn::parse_file(&content).unwrap();
    let formatted = prettyplease::unparse(&file);
    insta::assert_snapshot!(formatted);
}
