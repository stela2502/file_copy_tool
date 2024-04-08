use std::env;
use file_copy_tool::revert_links;

fn main() {
    let args: Vec<String> = env::args().collect();
    let hepl_str = "Links matching the pattern in the <folder> will be replaced by the real data.\n".to_string() +
        	"The copy process will be checked using sha256 hashes.\n"+ &format!("\nUsage: {} <folder>  <pattern1> [<pattern2> ...]", &args[0]);
    if args.len() < 3 {
        println!("{}", hepl_str );
        return;
    }
    let folder = &args[1];
    let patterns = &args[2..];

    revert_links(folder, patterns);
}

