//test_lib.rs

#[cfg(test)]
mod tests {
    use file_copy_tool::calculate_sha256;
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_calculate_sha256() {
    	let source_folder = "tests/source";

        if Path::new(&source_folder).exists(){
            fs::remove_dir_all(source_folder).unwrap();
        }

        fs::create_dir_all(source_folder).unwrap();

        let source_files = vec![ Path::new("tests/source/fileA.txt"), Path::new("tests/source/fileB.txt") ];

        for test_file in &source_files {
            fs::write(test_file, "Test content").unwrap();
        }

        let hash_a = calculate_sha256(&source_files[0]).unwrap_or_else(|err| {
        panic!("Error calculating SHA-256 hash for file {}: {}", source_files[0].display(), err)
	    });

	    let hash_b = calculate_sha256(&source_files[1]).unwrap_or_else(|err| {
	        panic!("Error calculating SHA-256 hash for file {}: {}", source_files[1].display(), err)
	    });

	    assert_eq!(hash_a, hash_b, "Hashes are the same: {} vs {}", hash_a, hash_b);

	    let different = Path::new("tests/source/fileC.txt");
	    fs::write(different, "Test different content").unwrap();

	    let hash_c = calculate_sha256(different).unwrap_or_else(|err| {
	        panic!("Error calculating SHA-256 hash for file {}: {}", different.display(), err)
	    });

	    assert_ne!(hash_a, hash_c, "Hashes are different: {} {}", hash_a, hash_c);
    }
}