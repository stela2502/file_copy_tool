use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn copy_files_matching_patterns(source_folder: &str, target_folder: &str, patterns: &[String]) {
    for entry in WalkDir::new(source_folder).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            for pattern in patterns {
                if file_name.ends_with(pattern) {
                    let target_path = Path::new(target_folder).join(file_path.strip_prefix(source_folder).unwrap());
                    if !target_path.exists() {
                        if let Some(parent_dir) = target_path.parent() {
                            fs::create_dir_all(parent_dir).unwrap();
                        }
                        fs::copy(file_path, &target_path).unwrap();
                        println!("Copied '{:?}' to '{:?}'", file_path, target_path);
                    } else {
                        println!("File '{:?}' already exists, skipping.", target_path);
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: {} <source_folder> <target_folder> <pattern1> [<pattern2> ...]", &args[0]);
        return;
    }
    let source_folder = &args[1];
    let target_folder = &args[2];
    let patterns = &args[3..];

    copy_files_matching_patterns(source_folder, target_folder, patterns);
}

