use std::env;
use std::fs;
use std::path::Path;
use std::io::{self, Read};
use std::os::unix::fs::symlink;
use walkdir::WalkDir;
use sha2::Digest;
use sha2::Sha256;
use hex::encode as hex_encode; // Add this line

fn calculate_sha256(file_path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    
    loop {
        let mut buffer = [0; 1024 * 1024]; // 1 MB buffer
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let result = hasher.finalize();
    let hash_bytes = result.as_slice();
    let hash_hex = hex_encode(hash_bytes);
    Ok(hash_hex)
}

fn copy_files_matching_patterns(source_folder: &str, target_folder: &str, patterns: &[String]) {
    for entry in WalkDir::new(source_folder).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();

            // Check if the file is a symbolic link
            if let Ok(metadata) = fs::metadata(&file_path) {
                if metadata.file_type().is_symlink() {
                    println!("Skipping symbolic link '{:?}'", file_path);
                    continue;
                }
            }

            for pattern in patterns {
                if file_name.ends_with(pattern) {
                    let target_path = Path::new(target_folder).join(file_path.strip_prefix(source_folder).unwrap());
                    if !target_path.exists() {
                        if let Some(parent_dir) = target_path.parent() {
                            fs::create_dir_all(parent_dir).unwrap();
                        }
                        fs::copy(&file_path, &target_path).unwrap();
                        println!("Copied '{:?}' to '{:?}'", file_path, target_path);

                        // Calculate MD5 hashes of source and target files
                        let source_md5 = calculate_sha256(&file_path).unwrap();
                        let target_md5 = calculate_sha256(&target_path).unwrap();
                        
                        // Compare MD5 hashes
                        if source_md5 == target_md5 {
                            // here we can savely remove the old file from the folder
                            println!("MD5 hashes match. Files are copied safely - removing the source version and replacing it with a link");
                            //println!("MD5 hashes match. Files are copied safely.");
                        } else {
                            eprintln!("MD5 hashes don't match. Files may not be copied correctly.");
                            if let Err(err) = fs::remove_file(&target_path) {
                                eprintln!("Error removing faulty file: {}", err);
                                continue;
                            }else {
                                eprintln!("Problematic copy has been removed.");
                            }

                        }
                    } else {
                        println!("File '{:?}' already exists, replacing with a symbolic link.", target_path);
                        if let Ok(abs_target) = fs::canonicalize(&target_path) {
                            if let Err(err) = fs::remove_file(&file_path) {
                                eprintln!("Error removing file: {}", err);
                                continue;
                            }
                            if let Err(err) = symlink(&abs_target, &file_path) {
                                eprintln!("Error creating symlink: {}", err);
                                continue;
                            }
                        }else {
                            panic!("I could not get the abs path for {:?}!",file_path);
                        }
                        println!("Created symbolic link for '{:?}' at '{:?}'", file_path, target_path);
                    }
                    break; // Stop processing of one file after copying
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


