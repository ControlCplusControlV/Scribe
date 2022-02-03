pub fn parse_value (string: &str) -> Option<&str> {
    let mut scanner = Scanner::new(string);

    scanner.transform(|character| match character {
        '$' => Some(string),
        '#' => Some(string),
        _ => None
    })
}


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

    /// Returns the current cursor. Useful for reporting errors.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the next character without advancing the cursor.
    /// AKA "lookahead"
    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    /// Returns true if further progress is not possible.
    pub fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    /// Invoke `cb` once. If the result is not `None`, return it and advance
/// the cursor. Otherwise, return None and leave the cursor unchanged.
pub fn transform<T>(
    &mut self,
    cb: impl FnOnce(&char) -> Option<T>,
    ) -> Option<T> {
    match self.characters.get(self.cursor) {
        Some(input) => match cb(input) {
            Some(output) => {
                self.cursor += 1;

                Some(output)
            },
            None => None
        },
        None => None
        }
    }
}




#[test]
fn test_parse_value(){
    let res = parse_value("$");
    let unwrapped_res = res.unwrap();
    println!("{}",unwrapped_res);

}