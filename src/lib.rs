//lib.rs

use std::fs;
use std::path::Path;
use std::io::{self, Read};
use std::os::unix::fs::symlink;
use walkdir::WalkDir;
use sha2::Digest;
use sha2::Sha256;
use hex::encode as hex_encode; // Add this line


pub fn calculate_sha256(file_path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    
    let mut buffer = [0; 1024 * 1024]; // 1 MB buffer
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read > 0{
            hasher.update(&buffer[..bytes_read]);
        } else {
            break;
        }
    }
    
    let result = hasher.finalize();
    let hash_bytes = result.as_slice();
    let hash_hex = hex_encode(hash_bytes);
    Ok(hash_hex)
}

pub fn copy_files_matching_patterns(source_folder: &str, target_folder: &str, patterns: &[String]) {
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
                            //println!("MD5 hashes match. Files are copied safely - removing the source version and replacing it with a link");
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
                        println!("File '{:?}' already exists, replacing the source with a symbolic link to it.", target_path);
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
                        println!("Created symbolic link for '{:?}' at '{:?}'", target_path, file_path);
                    }
                    break; // Stop processing of one file after copying
                }
            }
        }
    }
}


pub fn revert_links(target_folder: &str, patterns: &[String]) {
    // Walk through the target folder
    for entry in WalkDir::new(target_folder).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            if let Ok(metadata) = fs::metadata(&file_path) {
                if !metadata.file_type().is_symlink() {
                    continue;
                }
            }
            for pattern in patterns {
                if file_name.ends_with(pattern) {
                    // Check if the entry is a symbolic link
                    if let Ok(metadata) = fs::symlink_metadata(&file_path) {
                        if metadata.file_type().is_symlink() {
                            // Get the target of the symbolic link
                            if let Ok(target_path) = fs::read_link(&file_path) {
                                // Replace the symbolic link with the target file
                                if let Err(err) = fs::remove_file(&file_path) {
                                    eprintln!("Failed to remove symbolic link {}: {}", file_path.display(), err);
                                    continue;
                                }
                                if let Err(err) = fs::copy(&target_path, &file_path) {
                                    eprintln!("Failed to copy {} to {}: {}", target_path.display(), file_path.display(), err);
                                    // Re-establish the symbolic link if the copy fails
                                    if let Err(link_err) = symlink(&target_path, &file_path) {
                                        eprintln!("Failed to re-establish symbolic link {}: {}", file_path.display(), link_err);
                                    }
                                    continue;
                                }
                                let source_md5 = calculate_sha256(&target_path).unwrap();
                                let target_md5 = calculate_sha256(&file_path).unwrap();
                                if source_md5 != target_md5 {
                                    if let Err(err) = fs::remove_file(&file_path) {
                                        eprintln!("Error removing faulty file: {}", err);
                                        continue;
                                    }
                                    if let Err(link_err) = symlink(&target_path, &file_path) {
                                        eprintln!("Failed to re-establish symbolic link {}: {}", file_path.display(), link_err);
                                    }
                                    continue;
                                }
                                println!("Reverted the process for {}: Removed symbolic link and replaced it with a copy of the target file.", file_path.display());
                            }
                        }
                    } else {
                        eprintln!("Failed to get metadata for {}", file_path.display());
                    }
                }
            }
        }
    }
}