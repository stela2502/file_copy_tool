//test_lib.rs

#[cfg(test)]
mod tests {

    use tempdir::TempDir;
    use file_copy_tool::*;
    use std::fs;    
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_calculate_sha256_def() {
        // Create a temporary file with some content
        let tmp_dir = tempdir::TempDir::new("test_dir").unwrap();
        let file_path = tmp_dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Test data").unwrap();

        // Calculate SHA256 hash of the file
        let hash = calculate_sha256(&file_path, &mut [0; 1024 * 1024]).unwrap();

        // Expected SHA256 hash of the content
        let expected_hash = "e27c8214be8b7cf5bccc7c08247e3cb0c1514a48ee1f63197fe4ef3ef51d7e6f";

        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_calculate_sha256() {
    	let source_folder = "tests/source";
        let mut buffer = [0; 1024 * 1024]; // 1 MB buffer

        if Path::new(&source_folder).exists(){
            fs::remove_dir_all(source_folder).unwrap();
        }

        fs::create_dir_all(source_folder).unwrap();

        let source_files = vec![ Path::new("tests/source/fileA.txt"), Path::new("tests/source/fileB.txt") ];

        for test_file in &source_files {
            fs::write(test_file, "Test content").unwrap();
        }

        let hash_a = calculate_sha256(&source_files[0], &mut buffer).unwrap_or_else(|err| {
        panic!("Error calculating SHA-256 hash for file {}: {}", source_files[0].display(), err)
	    });

	    let hash_b = calculate_sha256(&source_files[1], &mut buffer).unwrap_or_else(|err| {
	        panic!("Error calculating SHA-256 hash for file {}: {}", source_files[1].display(), err)
	    });

	    assert_eq!(hash_a, hash_b, "Hashes are the same: {} vs {}", hash_a, hash_b);

	    let different = Path::new("tests/source/fileC.txt");
	    fs::write(different, "Test different content").unwrap();

	    let hash_c = calculate_sha256(different, &mut buffer).unwrap_or_else(|err| {
	        panic!("Error calculating SHA-256 hash for file {}: {}", different.display(), err)
	    });

	    assert_ne!(hash_a, hash_c, "Hashes are different: {} {}", hash_a, hash_c);

	    fs::remove_dir_all(source_folder).unwrap();
    }

    #[test]
    fn test_copy_file_with_hash_check() {
        // Create a temporary source file with some content
        let tmp_dir = tempdir::TempDir::new("test_dir").unwrap();
        let source_file_path = tmp_dir.path().join("source_file.txt");
        let mut source_file = File::create(&source_file_path).unwrap();
        source_file.write_all(b"Test data").unwrap();

        // Create a temporary target file
        let target_file_path = tmp_dir.path().join("target_file.txt");

        // Copy the file and check hash
        assert!(copy_file_with_hash_check(&source_file_path, &target_file_path, &mut [0; 1024 * 1024]).is_ok());

        // Check if the target file is created
        assert!(fs::metadata(&target_file_path).is_ok());

        // Copying again should return an error as the target file already exists
        assert!(copy_file_with_hash_check(&source_file_path, &target_file_path, &mut [0; 1024 * 1024]).is_err());
    }

    #[test]
    fn test_replace_with_symlink() {
        // Create a temporary source file with some content
        let tmp_dir = tempdir::TempDir::new("test_dir").unwrap();
        let source_file_path = tmp_dir.path().join("source_file.txt");
        let mut source_file = File::create(&source_file_path).unwrap();
        source_file.write_all(b"Test data").unwrap();

        // Create a temporary target file
        let target_file_path = tmp_dir.path().join("target_file.txt");

        // Replace the target file with a symlink
        match replace_with_symlink(&source_file_path, &target_file_path) {
            Some(_) => assert!(true,"as expedcted it worked?"),
            None => panic!("replace_with_symlink has come back None"),
        };
        // Check if the symlink is created correctly
        let symlink = fs::read_link(&target_file_path);
        assert!(symlink.is_ok());
        assert_eq!(symlink.unwrap(), source_file_path);
    }

}