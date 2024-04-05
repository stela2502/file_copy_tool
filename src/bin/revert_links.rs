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

fn revert_links(target_folder: &str, patterns: &[String]) {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <folder>  <pattern1> [<pattern2> ...]", &args[0]);
        return;
    }
    let folder = &args[1];
    let patterns = &args[2..];

    revert_links(folder, patterns);
}

