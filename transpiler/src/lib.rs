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

struct Context {
    variables: HashMap<String, u32>,
    next_open_memory_address: u32,
}

enum YulOp {
    Add(OpAdd),
    DeclareVariable(OpDeclareVariable),
}

struct OpDeclareVariable {
    identifier: String,
    value: u128,
}

struct OpAdd {
    firstVar: String,
    secondVar: String,
}

fn declare_var(program: &mut String, op: &OpDeclareVariable, context: &mut Context) {
    let address = context.next_open_memory_address;
    dbg!(&address);
    context.next_open_memory_address += 1;
    context.variables.insert(op.identifier.clone(), address);
    add_line(program, &format!("push.{}", op.value));
    add_line(program, &format!("mem.store.{}", address));
}

fn add(program: &mut String, op: &OpAdd, context: &mut Context) {
    for var in [&op.firstVar, &op.secondVar] {
        let address = context.variables.get(var).unwrap();
        add_line(program, &format!("mem.load.{}", address));
    }
    add_line(program, &format!("add"));
}

fn add_line(program: &mut String, line: &str) {
    *program = format!("{}\n{}", program, line)
}

#[test]
fn test_add() {
    let mut program = "begin\npush.0\npush.0\npush.0".to_string();
    let ops = vec![
        YulOp::DeclareVariable(OpDeclareVariable {
            identifier: "foo".to_string(),
            value: 12,
        }),
        YulOp::DeclareVariable(OpDeclareVariable {
            identifier: "bar".to_string(),
            value: 15,
        }),
        YulOp::Add(OpAdd {
            firstVar: "foo".to_string(),
            secondVar: "bar".to_string(),
        }),
    ];
    let mut context = Context {
        variables: HashMap::new(),
        next_open_memory_address: 0,
    };

    for op in ops {
        match op {
            YulOp::Add(op) => add(&mut program, &op, &mut context),
            YulOp::DeclareVariable(op) => declare_var(&mut program, &op, &mut context),
        }
    }
    add_line(&mut program, "end");

    println!("{}", program);
}
