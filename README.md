
<br />
<p align="center">
    <img src="Scribe.png" alt="Logo" width="800" height="450">
  <p align="center">

 
<br />
      
# 📜 Scribe 📜

Scribe is compact Yul transpiler written in Rust that targets the Polygon Miden VM. Scribe works by compiling Yul down to Miden opcodes, allowing developers to write smart contracts in Yul and run them on Polygon Miden. Since Yul is the intermediate language for Solidity, Vyper and Yul+ Scribe is a great foundation for various smart contract languages to compile code to run on Polygon Miden.

Currently, Scribe is able to transpile Yul syntax including blocks, for loops, if statements, break/continue, number literals, true literals, false literals, variable declarations, assignments, and function calls. We are still in the process of adding recognition for function definitions, switch/case statements, leave statements, string literals and hex numbers.


## How Does it Work?

Scribe is built with Rust and uses the [Pest parser](https://github.com/pest-parser/pest) to be able to recognize Yul Grammar. Scribe then translates the Yul code to the Miden VM opcodes, enabling fully functional Miden assembly can be generated from Yul. Since languages like Solidity and Vyper compile to Yul before generating EVM opcodes, in future versions of Scribe, it will be possible to transpile code written in Solidity or Vyper, into Miden assembly!
         
 <br/>
      
**Lets take a closer look at how Scribe works under the hood.**
      
      
First, Scribe reads in all of the Yul contracts in the `Scribe/contracts` directory. While Scribe can transpile entire Yul contracts, for this walkthrough we will just use a simple snippet of Yul code.
      
```js
      
object "fib" {
  code {
      
    let f := 1
    let s := 1
    let next
    
    for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    {
      if lt(i, 2) {
        mstore(i, 1)
      }
      
      if gt(i, 1) {
        next := add(s, f)
        f := s
        s := next
        mstore(i, s)
      }
      
    }
  }
}  
      
 ```
       
Once the Yul code is read in, Scribe converts the Yul code into a string and passes it into the `parse_yul_syntax` function. From there Scribe parses the string for Yul grammar, generating an enum for each match. 
      
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
            

      


## How Can I Test It?

Scribe was meant to focus on real world applicability, and because of this uses Miden v0.2. Due to Miden v0.2 not being done yet certain crates of it like the zk prover are broken atm as the developers build away on the new release. So certain functionality like zk proof generation can't be done at the moment, but execution can still be tested in the zk VM environment.

First clone this repo and download its submodule

```
git clone https://github.com/ControlCplusControlV/Scribe
```

then cd into the transpiler crate

```
cd transpiler
```

Then init the git submodule, and you should be good to go!

TODO Add instructions for people to compile and run their own Yul Programs
