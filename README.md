
<br />
<p align="center">
    <img src="Scribe.png" alt="Logo" width="800" height="450">
  <p align="center">

 
<br />
      
# 📜 Scribe 📜
![Testing](https://github.com/ControlCplusControlV/Scribe/actions/workflows/rust.yml/badge.svg)

Scribe is a compact Yul transpiler written in Rust that targets the Polygon
Miden VM. Scribe works by compiling Yul down to Miden opcodes, allowing
developers to write smart contracts in Yul and run them on Polygon Miden. Since
Yul is the intermediate language for Solidity, Vyper and Yul+ Scribe is a great
foundation for various smart contract languages to compile code to run on
Polygon Miden.

## Status

### Parsing

All yul syntax is parsed, including the new typed identifier list syntax.

Data blocks are not transpiled. Objects are naively transpiled as a series of statements.

### Types

Because u256 operations are so expensive in miden, scribe will check whether
variables and parameters are typed, and if they're `u32` values, then we can
use the much cheaper miden u32 operations. Scribe will default to `u256`.


### Supported yul functions

| Function | u32 | u256 | notes |
|----------|------|-----| ---- | 
| add      | ✅    | ✅ | |
| mul      |  ✅    |  ✅  | |
| sub      |   ✅   |  ✅   | |
| div      |   ✅   |  ❌   | |
| and      |   ✅   |  ✅   | |
| or      |   ✅   |  ✅   | |
| xor      |   ✅   |  ✅   | |
| mstore      |   ✅   |  ✅  | Only supports literals for the memory address |
| mload      |   ✅   |  ✅  | Only supports literals for the memory address |
| iszero      |   ✅   |  ✅  | |
| eq      |   ✅   |  ✅  | |
| lt      |   ✅   |  ✅  | |
| gt      |   ✅   |  ✅  | |
| gte      |   ✅   |  ✅  | |
| lte      |   ✅   |  ✅  | |
| shl      |   ✅   |  ✅  | |
| shr      |   ✅   |  ✅  | |


## Miden Repl

Scribe features a REPL to write miden assembly. You can start the repl with:

```
cargo run -- repl
```

From within the repl, you can write any valid miden assembly, then check the
stack with `stack`, or check your whole program with `program`. Anything that
errors out will not be added to the program. You can undo the last command with `undo`.

```
$ cargo run -- repl

>> push.4

>> push.5 push.7 mul

>> stack

  35 4 0 0 0 0 0 0 0 0 0 0 0 0 0 0

>> program

  begin
      push.4
      push.5 push.7 mul
  end

>> undo
  Undoing push.5 push.7 mul

>> stack

  4 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0

>> help

  Available commands:

  stack: display the stack
  undo: remove the last instruction
  program: display the program
```

## How Does it Work?

Scribe is built with Rust and uses the [Pest parser](https://github.com/pest-parser/pest) to be able to recognize Yul Grammar. Scribe then translates the Yul code to the Miden VM opcodes, enabling fully functional Miden assembly can be generated from Yul. Since languages like Solidity and Vyper compile to Yul before generating EVM opcodes, in future versions of Scribe, it will be possible to transpile code written in Solidity or Vyper, into Miden assembly!
         
 <br/>
      
**Lets take a closer look at how Scribe works under the hood.**
      
      
First, Scribe reads in all of the Yul contracts in the `Scribe/contracts` directory. While Scribe can transpile entire Yul contracts, for this walkthrough we will just use a simple snippet of Yul code.
      
```js
      
 object "fib" {
  code {
            let n := 10
            let a := 0
            let b := 1
            let c := 0

            for { let i := 0 } lt(i, n) { i := add(i, 1)}
            {
                c := add(a,b)
                a := b
                b := c
            }
            b
  }
}
      
 ```
       
Once the Yul code is read in, Scribe converts the code into a string and passes it into the `parse_yul_syntax` function. From there, Scribe parses the string, looking for Yul grammar and generates an `Expr` for each match.
      
 ```rust 
pub enum Expr {
    Literal(u128),
    FunctionCall(ExprFunctionCall),
    IfStatement(ExprIfStatement),
    Assignment(ExprAssignment),
    Gt(ExprGt),
    Lt(ExprLt),
    DeclareVariable(ExprDeclareVariable),
    ForLoop(ExprForLoop),
    Block(ExprBlock),
    Variable(ExprVariableReference),
}
```
      
Each `Expr` is pushed to a `Vec<Expr>`, which is then passed into the `miden_generator::transpile_program()` function. This function generates the Miden opcodes and keeps track of the variables as well as open memory addresses.  
      
```rust
pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut context = Context {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 4,
    };
    let mut program = "begin".to_string();
    for expr in expressions {
        transpile_op(&expr, &mut program, &mut context);
    }
    context.indentation -= 4;
    add_line(&mut program, "end", &context);
    return program;
}

fn transpile_op(expr: &Expr, program: &mut String, context: &mut Context) {
    match expr {
        Expr::Literal(value) => insert_literal(program, *value, context),
        Expr::Assignment(op) => assignment(program, op, context),
        Expr::DeclareVariable(op) => declare_var(program, op, context),
        Expr::ForLoop(op) => for_loop(program, op, context),
        Expr::Variable(op) => load_variable(program, op, context),
        Expr::Block(op) => block(program, op, context),
        Expr::IfStatement(op) => if_statement(program, op, context),
        Expr::FunctionCall(op) => {
            if (op.function_name == "add") {
                add(program, op, context)
            } else if (op.function_name == "gt") {
                gt(program, op, context)
            } else if (op.function_name == "lt") {
                lt(program, op, context)
            } else {
                todo!("Need to implement {} function in miden", op.function_name)
            }
        }
        x => todo!("{:?} unimplemented", x),
    }
}
```
      
The transpiled code from the fibonacci example looks something like this.
      
```nasm
      
begin
push.10
push.0
push.0
push.0
mem.pop.0
      
      
ect...
   
```
      
Now the generated Miden code is ready to run! Scribe can test your code on the Miden VM by starting the VM, passing in the Miden code and calling the executor. Here is what the process looks like from start to finish!
      
```rust
//Parse the Yul code
let parsed_yul_code = parser::parse_yul_syntax(yul_code);

//Generate Miden opcodes from the parsed Yul code
let miden_code = miden_generator::transpile_program(parsed);

//Execute the Miden code on the Miden VM
let execution_value = executor::execute(miden_code, inputs).unwrap();
let stack = execution_value.last_stack_state();
let last_stack_value = stack.first().unwrap();

//Print the output
println!("Miden Output");
println!("{}", last_stack_value);
```
      
And here is the output!
      
```
Miden Output
89    
```
      
      
      
## How to transpile your own contract.

To transpile and test your own contracts simple drop your own Yul Contracts inside the contracts folder then transpile then by running the transpiler crate with `cargo run`. Note that some Yul operations are still unsupported, but basic arithmatic, and control structures are supported, as well as variables.   

Scribe was meant to focus on real world applicability, and because of this uses Miden v0.2. Due to Miden v0.2 not being done yet certain crates of it like the zk prover are broken atm as the developers build away on the new release. So certain functionality like zk proof generation can't be done at the moment, but execution can still be tested in the zk VM environment.

First clone this repo and download its submodule

```
git clone https://github.com/ControlCplusControlV/Scribe
cd Scribe
git submodule init
git submodule update
```

then cd into the transpiler crate

```
cd transpiler
```

Then init the git submodule, and you should be good to go!
      
      
## Contributing

### Testing

Scribe has unit tests and integration tests that can be run with `cargo test`

Our parsing tests use the [insta](https://github.com/mitsuhiko/insta)
snapshot-testing crate. After running a new test, run `cargo insta review`,
verify that the generated AST looks right, then accept the output as correct.
In future tests the output will be compared to this snapshot.
