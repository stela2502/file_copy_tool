//revert_links_test.rs


#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    
    #[test]
    fn test_revert_links() {
    	// Set up test data and create the links
        let source_folder = "tests/source";
        let target_folder = "tests/target";

        if Path::new(&source_folder).exists(){
            fs::remove_dir_all(source_folder).unwrap();
        }
        if Path::new(&target_folder).exists(){
            fs::remove_dir_all(target_folder).unwrap();
        }

        let source_folders = vec![ "tests/source/folderA", "tests/source/folderA/subF", "tests/source/folderB"];

        fs::create_dir_all(source_folder).unwrap();
        fs::create_dir_all(target_folder).unwrap();
        for subf in source_folders {
            fs::create_dir_all(subf).unwrap();
        }

        let source_files = vec![ "tests/source/fileA.txt", "tests/source/fileB.txt", "tests/source/folderA/subF/fileC.txt",
            "tests/source/fileC.not", "tests/source/fileC.TXT", "tests/source/folderA/subF/fileC.not"
        ];

        for test_file in source_files {
            fs::write(test_file, "Test content").unwrap();
        }

        let good = vec![ "tests/target/fileA.txt", "tests/target/fileB.txt", "tests/target/folderA/subF/fileC.txt"];
        let bad = vec!["tests/target/fileC.not", "tests/target/fileC.TXT", "tests/target/folderB", "tests/target/folderA/subF/fileC.not"];

        
        // Run the file copying tool as a subprocess

        let is_release_mode = !cfg!(debug_assertions);

        let output = Command::new(
    	if is_release_mode { "./target/release/file_copy_tool" } 
    	else { "./target/debug/file_copy_tool" }
    	).args(&[source_folder, target_folder,".txt"])
        .output()
        .expect("Failed to execute command");
        
        // Check if the tool exited successfully
        assert!(output.status.success());

        // Check if the file is copied and its hash matches

        for copied_file in &good {
        	assert!(Path::new(copied_file).exists(), "file {copied_file} has not been copied?!");
        	if let Ok(metadata) = fs::metadata(copied_file) {
                assert!( ! metadata.file_type().is_symlink(), "target file {copied_file} is a symlink?!")
            }
            let source_file = copied_file.replace("tests/target", "tests/source");
            if let Ok(metadata) = fs::metadata(&source_file) {
                assert!( ! metadata.file_type().is_symlink(), "source file {copied_file} is a symlink after the first round?!")
            }
        }

        for copied_file in bad {
        	assert!( ! Path::new(&copied_file).exists(), "file {copied_file} has been copied but I did not ask for that?!");
        }

        // re-run of the tool fixing the symlinks

        let _output = Command::new(
        if is_release_mode { "./target/release/file_copy_tool" } 
        else { "./target/debug/file_copy_tool" }
        ).args(&[source_folder, target_folder,".txt"])
        .output()
        .expect("Failed to execute command");

        for copied_file in &good {
            assert!(Path::new(copied_file).exists(), "file {copied_file} has not been copied?!");
            if let Ok(metadata) = fs::metadata(copied_file) {
                assert!( ! metadata.file_type().is_symlink(), "target file {copied_file} is a symlink?!")
            }
            
            let source_file = copied_file.replace("tests/target", "tests/source");
            match fs::symlink_metadata(&source_file) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() {
                        assert!(true, "this is a link"); // just to get the count up
                    }else {
                        panic!("{} is not a link", source_file);
                    }
                },
                Err(_err) => panic!("Failed to get metadata for {source_file} is the link to {copied_file} broken?!"), //that is ok as a symlink does somehow not get the fs::metadata...
            };
        }

        // now the situation is set up to have all the links that I now want to reset to the original file

        let _output = Command::new(
        if is_release_mode { "./target/release/revert_links" } 
        else { "./target/debug/revert_links" }
        ).args(&[source_folder, ".txt"])
        .output()
        .expect("Failed to execute command");

        for copied_file in &good {
        	assert!(Path::new(copied_file).exists(), "file {copied_file} has not been copied?!");
        	if let Ok(metadata) = fs::symlink_metadata(copied_file) {
                assert!( ! metadata.file_type().is_symlink(), "target file {copied_file} is a symlink?!")
            }
            let source_file = copied_file.replace("tests/target", "tests/source");
            if let Ok(metadata) = fs::symlink_metadata(&source_file) {
                assert!( ! metadata.file_type().is_symlink(), "source file {copied_file} is still a symlink after revert_links?!")
            }
        }
        
        fs::remove_dir_all(target_folder).unwrap();
        fs::remove_dir_all(source_folder).unwrap();
    }
}