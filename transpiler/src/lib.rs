use std::collections::HashMap;

//Function to parse yul syntax and convert it into miden op codes
pub fn parse_value(string: &str) -> Option<&str> {
    let mut scanner = Scanner::new(string);

    //Yul syntax targets
    scanner.transform(|character| match character {
        '$' => Some(string),
        '#' => Some(string),
        _ => None,
    })
}

//Scanner struct that evaluates string to match specific target
pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

impl Scanner {
    pub fn new(string: &str) -> Self {
        Self {
            cursor: 0,
            characters: string.chars().collect(),
        }
    }

    /// Invoke `cb` once. If the result is not `None`, return it and advance
    /// the cursor. Otherwise, return None and leave the cursor unchanged.
    pub fn transform<T>(&mut self, cb: impl FnOnce(&char) -> Option<T>) -> Option<T> {
        match self.characters.get(self.cursor) {
            Some(input) => match cb(input) {
                Some(output) => {
                    self.cursor += 1;

                    Some(output)
                }
                None => None,
            },
            None => None,
        }
    }
}

//test parse value
#[test]
fn test_parse_value() {
    let res = parse_value("$");
    let unwrapped_res = res.unwrap();
    println!("{}", unwrapped_res);
}
