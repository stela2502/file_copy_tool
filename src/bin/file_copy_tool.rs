use std::env;

use file_copy_tool::copy_files_matching_patterns;



fn main() {
    let args: Vec<String> = env::args().collect();
    let help_str = "Identifies files using and end sting match and copies these files to the target_folder.\n".to_string()+
            "The copy wil be verified using sha256 hashes.\n"+
            "When run a second time the originals will be replaced by soft links instead.\n"+
            &format!("\nUsage: {} <source_folder> <target_folder> <pattern1> [<pattern2> ...]\n", &args[0]);
    if args.len() < 4 {
        println!( "{}", help_str );
        return;
    }
    let source_folder = &args[1];
    let target_folder = &args[2];
    let patterns = &args[3..];

    copy_files_matching_patterns(source_folder, target_folder, patterns);
}


