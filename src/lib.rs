//lib.rs

use std::fs;
use std::path::Path;
use std::io::{self, Read};
use walkdir::WalkDir;
use sha2::Digest;
use sha2::Sha256;
use hex::encode as hex_encode; // Add this line

#[cfg(windows)]
use std::os::windows::fs::{symlink_file};
#[cfg(unix)]
use std::os::unix::fs::{symlink};


#[cfg(windows)]
fn create_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    symlink_file(src, dst)
}

#[cfg(unix)]
fn create_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    symlink(src, dst)
}

pub fn copy_files_matching_patterns(source_folder: &str, target_folder: &str, patterns: &[String]) {

    // the copy returns this error if the file alredy existed:

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
                    
                    match copy_file_with_hash_check(&file_path, &target_path) {
                        Ok(_) => (),
                        Err(ref err) if err.to_string() == "link target existed" => {
                            println!("repacing {:?} with a link to {:?}", &file_path, &target_path);
                            replace_with_symlink( &file_path, &target_path );
                        },
                        Err(err) => eprintln!("{}", err),
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
                // only process symlinks
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
            panic!("calculate_sha256 read {} bytes", bytes_read);
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
        println!("Calculate hash source");
        let source_md5 = match calculate_sha256(&source_path){
            Ok(v) => v,
            Err(err) => panic!("calculate_sha256 source hit a wall {err}"),
        };
        println!("Calculate hash target");
        
        let target_md5 = match calculate_sha256(&target_path){
            Ok(v) => v,
            Err(err) => panic!("calculate_sha256 target hit a wall {err}"),
        };

        if source_md5 != target_md5 {
            fs::remove_file(&target_path)?;
            return Err(io::Error::new(io::ErrorKind::Other, "MD5 hashes don't match. Files may not be copied correctly."));
        }
        Ok(())
    } else {
        println!("The target path exists already - not copying!");
        return Err(io::Error::new(io::ErrorKind::Other, "link target existed"));
    }
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

fn replace_with_symlink( file_2_replace: &Path, link_target: &Path) -> Option<()> {

    // Create the symlink
    if let Ok(abs_target) = fs::canonicalize(&link_target) {

        // Rename the file to be replaced
        let renamed_file = file_2_replace.with_extension("bak");
        if let Err(err) = fs::rename(&file_2_replace, &renamed_file) {
            panic!("Error renaming file: {}", err);
        }
        if let Err(err) = create_symlink(&abs_target, &file_2_replace) {
            eprintln!("Error creating symlink: {}", err);
            
            if let Err(err) = fs::rename(&renamed_file ,&file_2_replace ) {
                // rename the file back to it's original name 
                panic!("Error renaming file: {}", err);
            }           
            None
        }else {
            // cool the renamed is not necessary any more
            if let Err(remove_err) = fs::remove_file(&renamed_file) {
                eprintln!("Error removing renamed file: {}", remove_err);
            }
            println!("Created symbolic link for '{:?}' at '{:?}'", link_target, file_2_replace);
            Some(())
        } 
        
    } else {
        eprintln!("Failed to get the absolute path for {:?}", link_target);
        None
    }
}
