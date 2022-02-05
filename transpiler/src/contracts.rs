use std::env;
use std::fs;

fn read_yul_files() -> Vec<String> {
    let mut yul_files: Vec<String> = Vec::new();

    let file_path = fs::read_dir("../contracts/").unwrap();

    let mut parsed_files: [String; 32] = Default::default();

    for file in file_path {
        let mut unwrapped_file = file.unwrap().path().display().to_string();
        let mut contents = fs::read_to_string(unwrapped_file)
        .expect("Something went wrong readingfrom the contracts directory");
        
        contents = contents.replace('\n', "");
        
        yul_files.push(contents);
    }
return yul_files
}

