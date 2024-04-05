[![Rust](https://github.com/stela2502/file_copy_tool/actions/workflows/rust.yml/badge.svg?branch=main2)](https://github.com/stela2502/file_copy_tool/actions/workflows/rust.yml)

# file_copy_tool

This is a simple recursive file copy tool that allows to copy any kind of file from a source folder to a target folder,
while keeping the relative path location of the file.

In the first pass this tool will simply copy the files checking the sha256 hash before and after.
If a faulty copy is detected the copied file will be removed again.

If you **run this tool a second time** and it detects files in both the source and the target folder the tool
**will remove the source** and replace with a soft link to the target instead!

# Install

```
git clone git@github.com:stela2502/file_copy_tool.git

cd file_copy_tool

cargo test -r

cp target/release/file_copy_tool ~/bin/

```

# Usage

```
file_copy_tool <source_folder> <target_folder> <pattern1> [<pattern2> ...]
```

The patterns are not really RegExp patterns but simply file end strings. So in order to e.g. copy all fastq.gz files from my_work_area to my_backup_area you would run this:

```
file_copy_tool my_work_area my_backup_area '.fastq.gz'
```

If you run this a second time the tool will remove all fastq.gz files from the my_work_area and replace them with soft links to the my_backup_area files. To save space you should run it a second time. But you can check if the backup data is OK before you do so.




