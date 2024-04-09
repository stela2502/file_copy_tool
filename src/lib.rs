//lib.rs

use std::fs;
use std::path::Path;
use std::io::{self, Read};
use std::os::unix::fs::symlink;
use walkdir::WalkDir;
use sha2::Digest;
use sha2::Sha256;
use hex::encode as hex_encode; // Add this line

pub fn copy_files_matching_patterns(source_folder: &str, target_folder: &str, patterns: &[String]) {
    for entry in WalkDir::new(source_folder).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            if let Ok(metadata) = fs::metadata(&file_path) {
                if metadata.file_type().is_symlink() {
                    println!("Skipping symbolic link '{:?}'", file_path);
                    continue;
                }
            }

            for pattern in patterns {
                if file_name.ends_with(pattern) {
                    let target_path = Path::new(target_folder).join(file_path.strip_prefix(source_folder).unwrap());
                    if let Err(err) = copy_file_with_hash_check(&file_path, &target_path) {
                        eprintln!("{}", err);
                    }
                    break; // Stop processing of one file after copying
                }
            }
        }
    }
}

pub fn revert_links(target_folder: &str, patterns: &[String]) {
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
                    if let Err(err) = revert_symlink(&file_path) {
                        eprintln!("{}", err);
                    }
                }
            }
        }
    }
}

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

fn copy_file_with_hash_check(source_path: &Path, target_path: &Path) -> io::Result<()> {
    if !target_path.exists() {
        if let Some(parent_dir) = target_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }
        fs::copy(source_path, target_path)?;
        println!("Copied '{:?}' to '{:?}'", source_path, target_path);

        // Calculate and compare hashes
        let source_md5 = calculate_sha256(&source_path)?;
        let target_md5 = calculate_sha256(&target_path)?;

        if source_md5 != target_md5 {
            fs::remove_file(&target_path)?;
            return Err(io::Error::new(io::ErrorKind::Other, "MD5 hashes don't match. Files may not be copied correctly."));
        }
    } else {
        symlink_and_replace(&target_path, &source_path);
    }
    Ok(())
}

fn revert_symlink(file_path: &Path) -> io::Result<()> {
    if let Ok(target_path) = fs::read_link(&file_path) {
        fs::remove_file(&file_path)?;
        if let Err(err) = copy_file_with_hash_check(&target_path, &file_path){
        	eprintln!("{}", err);
        }
        println!("Reverted the process for {}: Removed symbolic link and replaced it with a copy of the target file.", file_path.display());
    }
    Ok(())
}

fn symlink_and_replace(target_path: &Path, file_path: &Path) {
    println!("File '{:?}' already exists, replacing the source with a symbolic link to it.", file_path);
    if let Ok(abs_target) = fs::canonicalize(&target_path) {
        if let Err(err) = fs::remove_file(&file_path) {
            eprintln!("Error removing file: {}", err);
            return;
        }
        if let Err(err) = symlink(&abs_target, &file_path) {
            eprintln!("Error creating symlink: {}", err);
            return;
        }
        println!("Created symbolic link for '{:?}' at '{:?}'", target_path, file_path);
    } else {
        panic!("Failed to get the absolute path for {:?}", file_path);
    }
}
