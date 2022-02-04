use std::env;
use std::fs;

fn read_yul_files() {
    let paths = fs::read_dir("./contracts/").unwrap();

    let mut parsed_files: [String; 32] = Default::default();

    let mut counter:usize = 0;
    for path in paths {
        let mut current_file_name = path.unwrap().path().display().to_string();

        println!("In file {}", current_file_name);

        let contents = fs::read_to_string(current_file_name)
        .expect("Something went wrong reading files from the contracts directory");
        
        parsed_files[counter] = contents;
        counter = counter + 1;
        println!("With text:\n{}", parsed_files[0]);
    }

}

