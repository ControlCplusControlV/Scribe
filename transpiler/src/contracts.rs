use std::env;
use std::fs;

pub struct YulFile{
    pub file_name: String,
    pub file_contents: String,
}

fn read_yul_files() -> Vec<YulFile> {
    let mut yul_files: Vec<YulFile> = Vec::new();

    let file_path = fs::read_dir("../contracts/").unwrap();

    for file in file_path {
        //TODO: get the file name

        let mut unwrapped_file = file.unwrap().path().display().to_string();
        let mut contents = fs::read_to_string(unwrapped_file)
        .expect("Something went wrong readingfrom the contracts directory");
        contents = contents.replace('\n', "");
        
        yul_files.push(YulFile{file_name:"", file_contents: contents});
    }
return yul_files
}

