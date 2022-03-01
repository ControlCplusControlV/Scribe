use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};

use primitive_types::U256;

use crate::{ast_optimization::optimize_ast, types::*, utils::convert_u256_to_pushes};

//Struct that enables transpilation management. Through implementations, this struct keeps track of the variables,
//open memory addresses, the stack, indentation of Miden assembly and user defined functions.
struct Transpiler {
    variables: HashMap<TypedIdentifier, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
    branches: VecDeque<Branch>,
    accept_overflow: bool,
}

//A branch is a temporary represnetation of a stack to keep track of where variables are
// in the Miden stack during conditional Statements. For example if you have a snippet of Yul code like this:
//let x := 10
//if gt(x,5) {
//    x = 5
// }
//The Miden assembly will look like this:
//begin
//   # push x to the stack #
//   push.10
//   push.5
//   gt
//   if.true
//       # Push the new value of x to the stack #
//       push.5
//Before the evaluating the if statement, the stack is [5, 10] with x being at index 1.
//If the if statement is true, then the stack would look like [5, 5, 10], and now the transpiler does not know where x is in the stack.
//A Branch is created during each if statement allowing the transpiler to keep track of the state of the stack before the if statement
// and restore the original state with the updated values. To do this, the transpiler looks at the stack_before in reverse and pushes
// the most recent value of each variable in the branch to the stack, effectively recreating the stack with the updated values.
#[derive(Default, Clone)]
struct Branch {
    modified_identifiers: HashSet<TypedIdentifier>,
    stack_before: Stack,
}

//Struct to represent a stack value.
//Each stack value has an optional typed identifier (ie. x or x:u256) and a YulType
#[derive(Clone, Debug)]
struct StackValue {
    typed_identifier: Option<TypedIdentifier>,
    yul_type: YulType,
}

impl From<&TypedIdentifier> for StackValue {
    fn from(typed_identifier: &TypedIdentifier) -> Self {
        StackValue {
            typed_identifier: Some(typed_identifier.clone()),
            yul_type: typed_identifier.yul_type,
        }
    }
}

//Struct to represent the stack, consisting of a Vec of Stack Values
#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

// Implementation to print the stack in the terminal
impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n").unwrap();
        for value in self.0.iter() {
            write!(
                f,
                "{}:{}\n",
                value
                    .typed_identifier
                    .clone()
                    .map(|ti| ti.identifier)
                    .unwrap_or("UNKNOWN".to_string()),
                value.yul_type
            )
            .unwrap();
        }
        Ok(())
    }
}

//Implementations to manage the transpilation process to Miden assembly.
//Ex. Stack management, memory management, branching, indendtation, adding comments
impl Transpiler {
    fn equate_reference(&mut self, x: TypedIdentifier, y: TypedIdentifier) {
        panic!("This no longer works, should fix for optimizations");
        // if x.yul_type != y.yul_type {
        //     panic!("Should never be assigning a {:?} to a {:?}", x, y);
        // }
        // let stack_value = self.stack.0.iter_mut().find(|sv| sv.contains(&y)).unwrap();
        // stack_value.insert(x);
    }

    //Function to indent the Miden assembly by four spaces.
    //Ex.
    // if.true
    //     push.0
    fn indent(&mut self) {
        self.indentation += 4;
    }

    //Function to decrease the indent in the Miden assembly by four spaces
    //Ex.
    // if.true
    //     push.0
    // drop
    fn outdent(&mut self) {
        self.indentation -= 4;
    }

    //Function to return the state of the stack before branching. For more details on a branch, see the Branch struct.
    fn target_stack(&mut self, target_stack: Stack) {
        for v in target_stack.0.iter().rev() {
            // TODO: can do a no-op or padding op if no identifiers
            self.push_identifier_to_top(
                v.typed_identifier
                    .clone()
                    .expect("Need to deal w/ this case"),
            );
        }
        self.stack.0 = self
            .stack
            .0
            .clone()
            .into_iter()
            .take(target_stack.0.len())
            .collect();
    }

    //Function to tell the transpiler to begin a branch during when entering a conditional statement
    fn begin_branch(&mut self) {
        self.branches.push_front(Branch {
            stack_before: self.stack.clone(),
            ..Default::default()
        });
    }

    //Function to tell the transpiler to end a branch when exiting a conditional statement, updating the state of the stack and variables.
    fn end_branch(&mut self) {
        let branch = self.branches.pop_front().unwrap();
        self.add_comment("cleaning up after branch");
        self.indent();
        for modified_identifier in branch.modified_identifiers {
            if branch
                .stack_before
                .0
                .iter()
                .find(|sv| sv.typed_identifier == Some(modified_identifier.clone()))
                .is_none()
            {
                self.update_identifier_in_memory(modified_identifier)
            }
        }
        self.target_stack(branch.stack_before);
        self.outdent();
    }

    //When the stack size is greater than 16 elements, if the transpiler pushed the value directly on the stack
    //the 16th element would move to the 17th index, which is out of the stack frame making it inaccessible.
    //Instead, the transpiler will move the value at the 16th index to the transpiler's virtual memory to keep the stack at 16 elements.
    //This temporary virtual memory is only used during transpilation to keep track of variables outside of
    //the stack from and does not affect Miden memory.

    //Function to tell the transpiler to allow the stack size to exceed 16 elements
    fn begin_accepting_overflow(&mut self) {
        self.accept_overflow = true;
    }

    //Function to tell the transpiler not to allow the stack size to exceed 16 elements and save values into the transpiler's virtual memory.
    fn stop_accepting_overflow(&mut self) {
        self.accept_overflow = true;
    }

    //TODO:
    fn add_unknown(&mut self, yul_type: YulType) {
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type,
            },
        );
    }

    //Function to update the value of a variable stored in memory. First, the value of the variable is pushed to the top of the stack
    // where it is then saved into memory. Under the hood, pop_stack_value_to_memory makes sure that the right value is replaced.
    //See pop_stack_value_to_memory for more details.
    fn update_identifier_in_memory(&mut self, typed_identifier: TypedIdentifier) {
        self.push_identifier_to_top(typed_identifier);
        self.pop_top_stack_value_to_memory();
    }

    //Function to load a variable's value from memory and push it to the top of the stack
    //See push_from_memory_to_top_of_stack for more details on how differently typed variables are loaded and pushed.
    fn load_identifier_from_memory(&mut self, typed_identifier: TypedIdentifier) {
        self.add_comment(&format!(
            "push {} from memory to top of stack",
            typed_identifier.identifier
        ));
        let address = self.variables.get(&typed_identifier).cloned().unwrap();
        self.push_from_memory_to_top_of_stack(address, &typed_identifier.yul_type);
        self.stack.0.first_mut().unwrap().typed_identifier = Some(typed_identifier);
    }

    //Push a value from memory to the top of the stack.
    //If the element is a u32, it takes up one 32bit element of one memory address. Since each memory address has four 32bit elements,
    //dup and dropw are used to remove padding from the stack.
    //When the element is a u256, it takes up two addresses completely, and the two address can simply be pushed onto the stack, taking up eight elements.
    fn push_from_memory_to_top_of_stack(&mut self, address: u32, yul_type: &YulType) {
        match yul_type {
            YulType::U32 => {
                self.add_line(&format!("pushw.mem.{}", address));
                self.add_line("dup");
                self.add_line("dropw");
            }
            YulType::U256 => {
                self.add_line(&format!("pushw.mem.{}", address + 1));
                self.add_line(&format!("pushw.mem.{}", address + 0));
            }
        }
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type: *yul_type,
            },
        );
        self.newline();
    }

    //FIXME: Still needs comments

    //not pushing but duping to the top, offset is just where the value starts
    //and depending on the type it has to dup 8 elements
    fn push_identifier_to_top(&mut self, typed_identifier: TypedIdentifier) {
        let mut offset = 0;
        self.prepare_for_stack_values(&typed_identifier.yul_type);
        let stack_value = self
            .stack
            .0
            .iter()
            .find(|sv| {
                if sv.typed_identifier.as_ref() == Some(&typed_identifier) {
                    return true;
                }
                offset += sv.yul_type.miden_stack_width();
                return false;
            })
            .cloned();
        match stack_value {
            Some(stack_value) => {
                self.stack
                    .0
                    .insert(0, StackValue::from(&typed_identifier.clone()));
                self.add_comment(&format!(
                    "pushing {} to the top",
                    typed_identifier.identifier
                ));
                self.indent();
                self.dup_from_offset(offset, stack_value.yul_type);
                self.outdent()
            }

            //If it cant find the identifier in the stack it will load it from memory
            None => {
                self.load_identifier_from_memory(typed_identifier);
            }
        }
    }

    //TODO: explain the function and then explain what an offset is
    fn dup_from_offset(&mut self, offset: u32, yul_type: YulType) {
        match yul_type {
            YulType::U32 => {
                self.add_line(&format!("dup.{}", offset));
            }
            YulType::U256 => match offset + 7 {
                7 => {
                    self.add_line(&format!("dupw.1"));
                    self.add_line(&format!("dupw.1"));
                }
                11 => {
                    self.add_line(&format!("dupw.2"));
                    self.add_line(&format!("dupw.2"));
                }
                15 => {
                    self.add_line(&format!("dupw.3"));
                    self.add_line(&format!("dupw.3"));
                }
                o => {
                    for _ in (0..8) {
                        self.add_line(&format!("dup.{}", o));
                    }
                }
            },
        };
    }

    //Function to push a u32 value on the stack
    fn push(&mut self, value: U256) {
        self.prepare_for_stack_values(&YulType::U32);
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type: YulType::U32,
            },
        );
        self.add_comment(&format!("u32 literal {}", value));
        self.add_line(&format!("push.{}", value));
        self.newline();
    }

    //Function to check the stack size and determine whether or not the push will put values on the stack or into memory
    //See comments on accept_overflow for more details on when the transpiler uses memory vs the stack
    fn prepare_for_stack_values(&mut self, yul_type: &YulType) {
        if self.accept_overflow {
            return;
        }
        while self.get_size_of_stack() + yul_type.miden_stack_width() > 16 {
            self.add_comment(&format!(
                "stack would be too large after {}, popping to memory",
                yul_type,
            ));
            // dbg!(&self.stack);
            self.indent();
            self.pop_bottom_var_to_memory();
            self.outdent();
            // dbg!(&self.stack);
        }
    }

    //Function to remove the bottom variable from the stack and move it to memory.
    //Under the hood, this function moves the bottom value to the top of the stack and then saves it to memory.
    //See pop_top_stack_value_to_memory for more details.
    fn pop_bottom_var_to_memory(&mut self) {
        let stack_value = self.stack.0.last().cloned().unwrap();
        dbg!(&self.stack);

        let stack_above: Vec<StackValue> = self
            .stack
            .0
            .clone()
            .into_iter()
            .take(self.stack.0.len() - 1)
            .collect();
        let num_stack_values_above: u32 = stack_above
            .iter()
            .map(|sv| sv.yul_type.miden_stack_width())
            .sum();
        self.add_comment(&format!(
            "Moving {} to top of stack",
            stack_value.typed_identifier.as_ref().unwrap().identifier
        ));
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line(&format!("movup.{}", num_stack_values_above));
            }
            YulType::U256 => {
                match num_stack_values_above {
                    1 => {
                        self.add_line(&format!("movdn.8"));
                    }
                    8 => {
                        self.add_line(&format!("movupw.3"));
                        self.add_line(&format!("movupw.3"));
                    }
                    o => {
                        for _ in (0..8) {
                            self.add_line(&format!("movup.{}", o + 7));
                        }
                    }
                };
            }
        }
        self.stack.0.pop();
        self.stack.0.insert(0, stack_value.clone());
        self.pop_top_stack_value_to_memory();
    }

    //Function to remove the top stack value from the stack and save it into memory.
    //If the variable is already stored in memory, the transpiler will update the value. Else, the transpiler will get the next available
    // memory address. In the case of a u32, padw is used to push four 0s and one is dropped to pop the top word into memory.
    //If the value is a u256, mem.pop.address and mem.pop.address+1 can be used to pop two words into into memory
    fn pop_top_stack_value_to_memory(&mut self) -> u32 {
        let stack_value = self.stack.0.first().unwrap().clone();

        let address = match stack_value
            .typed_identifier
            .clone()
            .map(|typed_identifier| self.variables.get(&typed_identifier.clone()))
            .flatten()
        {
            Some(address) => *address,
            None => {
                let address = self.next_open_memory_address;
                if let Some(ref typed_identifier) = stack_value.typed_identifier {
                    self.variables.insert(typed_identifier.clone(), address);
                }
                self.next_open_memory_address += stack_value.yul_type.miden_memory_addresses();
                address
            }
        };
        self.add_comment(&format!(
            "popping {} from top of stack to memory",
            stack_value
                .typed_identifier
                .clone()
                .map(|ti| ti.identifier)
                .unwrap_or("unknown".to_string())
        ));
        self.stack.0.remove(0);
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("padw");
                self.add_line("drop");
                self.add_line(&format!("popw.mem.{}", address));
            }
            YulType::U256 => {
                self.add_line(&format!("popw.mem.{}", address));
                self.add_line(&format!("popw.mem.{}", address + 1));
            }
        }
        self.newline();
        address
    }

    //Function to get the next available memory address
    fn get_next_open_memory_address(&mut self) {
        self.next_open_memory_address;
    }

    //Return the size of the stack.
    fn get_size_of_stack(&self) -> u32 {
        self.stack
            .0
            .iter()
            .map(|sv| sv.yul_type.miden_stack_width())
            .sum()
    }

    //Push a u256 value to the stack. See convert_u256_to_pushes for more details on how u256 segments are pushed.
    fn push_u256(&mut self, value: U256) {
        self.prepare_for_stack_values(&YulType::U256);
        self.add_comment(&format!("u256 literal: {}", value));
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type: YulType::U256,
            },
        );
        self.add_line(&convert_u256_to_pushes(&value));
        self.newline();
    }

    //Consume n stack values from the top of the stack
    fn _consume_top_stack_values(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
    }

    //Duplicate the top stack value. If the YulType is u32, the first 32bit element is simply dupped.
    //If the Yultype is u256, dupw.1 is called twice.
    fn dup_top_stack_value(&mut self) {
        self.add_comment(&format!("duping top stack value"));
        let stack_value = self.stack.0.get(0).unwrap().clone();
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("dup");
            }
            YulType::U256 => {
                self.add_line("dupw.1");
                self.add_line("dupw.1");
            }
        }
        self.stack.0.insert(0, stack_value.clone());
        self.newline();
    }

    //Drop the top value from the stack. Similar to other functions, if the YulType is u32, drop is called.
    //If the YulType is u256, dropw is called twice
    fn drop_top_stack_value(&mut self) {
        self.add_comment(&format!("dropping top stack value"));
        let stack_value = self.stack.0.get(0).unwrap().clone();
        self.stack.0.remove(0);
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("drop");
            }
            YulType::U256 => {
                self.add_line("dropw");
                self.add_line("dropw");
            }
        }
        self.newline();
    }

    //Drops the stack after the nth element.
    //For example if the stack is [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    // drop_after(10) will result [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0]
    fn drop_after(&mut self, n: usize) {
        let size_to_keep: u32 = self
            .stack
            .0
            .iter()
            .take(n)
            .map(|sv| sv.yul_type.miden_stack_width())
            .sum();
        let total_size: u32 = self
            .stack
            .0
            .iter()
            .map(|sv| sv.yul_type.miden_stack_width())
            .sum();

        for n in 0..n {
            let stack_length = self.stack.0.len();
            self.stack.0.swap(n, stack_length - 1);
        }
        self.add_comment(&format!("dropping after {} ", n));
        for n in 0..size_to_keep {
            let shift = total_size - 1;
            if shift == 1 {
                self.add_line(&format!("swap"));
            } else {
                self.add_line(&format!("movdn.{}", shift));
            }
        }
        for i in (n..self.stack.0.len()).rev() {
            self.drop_top_stack_value();
        }
        self.newline();
    }

    //Sets the value at the top of the stack to a typed_identifier. For example, when a value gets pushed to the stack, it is unknown.
    //Then once top_is_var(TypedIdentifier:x, YulType:U32) is called, that value is assigned to the variable x.
    //Lets take a look starting from assigning a variable in Yul. let x := 10 The transpiler pushes 10 to the stack and then calls top_is_var(TypedIdentifier:x, YulType:U32)
    //to associate that value with the variable x
    fn top_is_var(&mut self, typed_identifier: TypedIdentifier) {
        let stack_value = self.stack.0.get_mut(0).unwrap();
        stack_value.typed_identifier = Some(typed_identifier);
    }

    //Modifies the stack to include values from developer written functions in Yul
    //For example, if someone were to write a function called return_two_numbers() that returns two values,
    // when that function is called, we have to modify our stack by pushing those two numbers to the top of the stack.
    fn add_function_stack(&mut self, function_stack: &Stack) {
        let mut new_stack = Stack::default();
        new_stack.0.append(&mut self.stack.0.clone());
        new_stack.0.append(&mut function_stack.0.clone());
        self.stack = new_stack;
    }

    //Get the data type for a specific variable
    fn get_typed_identifier(&self, identifier: &str) -> &TypedIdentifier {
        self.scoped_identifiers
            .get(identifier)
            .expect(&format!("\"{}\" not in scope", identifier))
    }
}

impl Transpiler {
    //Transpile a variable declaration.
    //Ex. let x := 1000 or let x:u256 := 1000
    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        assert_eq!(op.typed_identifiers.len(), 1);
        // let address = self.next_open_memory_address;
        for typed_identifier in &op.typed_identifiers {
            self.scoped_identifiers.insert(
                typed_identifier.identifier.clone(),
                typed_identifier.clone(),
            );
        }
        // self.next_open_memory_address += 1;
        // TODO: multiple declarations not working
        assert_eq!(op.typed_identifiers.len(), 1);
        // TODO: should use memory probably
        // self.variables.insert(op.identifier.clone(), address);
        self.add_comment(&format!(
            "Assigning to {}",
            op.typed_identifiers.first().unwrap().identifier
        ));
        self.indent();
        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs);
            self.top_is_var(op.typed_identifiers.first().unwrap().clone());
        }
        self.outdent();
    }

    //Transpile an assignment
    //Ex. x = 1000 or x = 1000:u256
    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let typed_identifier = self
            .get_typed_identifier(&op.identifiers.first().unwrap())
            .clone();
        self.add_comment(&format!("Assigning to {}", typed_identifier.identifier));
        self.indent();
        if let Some(branch) = self.branches.front_mut() {
            branch.modified_identifiers.insert(typed_identifier.clone());
        }
        // TODO: bring back equating references
        // if let Expr::Variable(ExprVariableReference {
        //     identifier: target_ident,
        // }) = &*op.rhs
        // {
        //     let typed_source_identifier = self.get_typed_identifier(target_ident);
        //     self.equate_reference(typed_identifier.clone(), typed_source_identifier.clone());
        // } else {
        self.transpile_op(&op.rhs);
        self.top_is_var(typed_identifier);
        self.outdent();
        // }
    }

    //Transpile a block. Loops through an ExprBlock which is a Vec of expressions and transpiles each expression.
    fn transpile_block(&mut self, op: &ExprBlock) {
        for op in &op.exprs {
            self.transpile_op(op);
        }
    }

    //Transpile a for loop. A for loop is made up of an init block, a conditional, an after block
    // and an interior block. Under the hood, while.true evaluates the top value of the stack so we must push the result of the conditional
    // after each loop. Here is an example of each block.
    //Ex.for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    // {
    //     if lt(i, 2) {
    //       mstore(i, 1)
    //      }
    //  }
    //Init block: for { let i := 0 }
    //Conditional: lt(i, 10)
    //After block: { i := add(i, 1)}
    //Interior Block: {if lt(i, 2) { mstore(i, 1)}}
    fn transpile_for_loop(&mut self, op: &ExprForLoop) {
        self.transpile_block(&op.init_block);
        self.add_comment("-- conditional --");
        self.transpile_op(&op.conditional);
        self.add_line("while.true");
        // Because the while.true will consume the top of the stack
        self._consume_top_stack_values(1);
        self.indent();
        self.begin_branch();

        self.add_comment("-- interior block --");
        self.indent();
        self.transpile_block(&op.interior_block);
        self.outdent();
        self.newline();

        self.add_comment("-- after block --");
        self.indent();
        self.transpile_block(&op.after_block);
        self.newline();
        self.end_branch();
        self.outdent();
        self.newline();

        self.add_comment("-- conditional --");
        self.indent();
        self.transpile_op(&op.conditional);
        self.outdent();
        self.newline();
        self._consume_top_stack_values(1);
        self.outdent();

        self.add_line("end");
        self.newline();
    }

    //Transpiles a repeat expression
    fn transpile_repeat(&mut self, op: &ExprRepeat) {
        let stack_target = self.stack.clone();
        self.add_line(&format!("repeat.{}", op.iterations));
        self.indent();
        self.transpile_block(&op.interior_block);
        self.target_stack(stack_target);
        self.outdent();
        self.add_line("end");
    }

    //Transpiles a switch statement
    //FIXME: Still needs comments
    fn transpile_switch(&mut self, op: &ExprSwitch) {
        self.add_line("");

        //bool for match
        let switch_matched_pseudovar = TypedIdentifier {
            identifier: "switch_matched".to_string(),
            yul_type: YulType::U32,
        };
        //set the bool in scoped variables
        self.scoped_identifiers.insert(
            switch_matched_pseudovar.identifier.clone(),
            switch_matched_pseudovar.clone(),
        );

        self.add_comment("keeping track of whether we've hit any cases");
        //push 0 on miden stack
        self.add_line("push.0");
        //push unknown on transpiler stack
        self.add_unknown(YulType::U32);
        //declare that top value in stack is the match psuedo var bool
        self.top_is_var(switch_matched_pseudovar.clone());

        //transpile the op which could be anything, pushes expr to the top (u32, u256)
        self.transpile_op(&op.expr);
        // TODO: these have to be unique, some way to do that, incrementing?

        //define case (expr) - not acutally defined, just to keep track of the variable
        let switch_target_pseudovar = TypedIdentifier {
            identifier: "switch".to_string(),
            yul_type: op.inferred_type.unwrap(),
        };

        //add case to scoped identifiers
        self.scoped_identifiers.insert(
            switch_target_pseudovar.identifier.clone(),
            switch_target_pseudovar.clone(),
        );

        //set the value at top of stack from transpile op to the case (from define case) pseudovar
        self.top_is_var(switch_target_pseudovar.clone());

        for (i, case) in op.cases.iter().enumerate() {
            self.push_identifier_to_top(switch_target_pseudovar.clone());

            //assumes that target is at the top of the stack, and will go into the branch if matches, transpiles branch if match
            //update match pseudovar (boolean) if match
            self.transpile_case(
                &case,
                &op,
                switch_target_pseudovar.clone(),
                switch_matched_pseudovar.clone(),
            );
        }
        if let Some(default_case) = &op.default_case {
            self.add_comment("default case");
            self.push_identifier_to_top(switch_matched_pseudovar);
            self.add_line("not");
            self._consume_top_stack_values(1);
            self.add_line("if.true");
            self.begin_branch();
            self.transpile_block(&default_case);
            self.end_branch();
            self.add_line("end");
        }
        self.add_line("");
    }

    //Transpiles a procedure in miden.
    //For example, if a function call is passed in as an expression, the first parameter type is u256 and the function name is "add"
    //a call to the procedure exec.u256add_unsafe will be added into the program.
    //If the first parameter is u256, the transpiler will use u256 operations
    fn transpile_miden_function(&mut self, op: &ExprFunctionCall) {
        for expr in op.exprs.clone().into_iter() {
            self.transpile_op(&expr);
        }

        self.add_comment(&format!("{}()", op.function_name));

        if let Some(function_stack) = self.user_functions.clone().get(&op.function_name) {
            self.add_line(&format!("exec.{}", op.function_name));
            self.add_function_stack(function_stack);
            return;
        }

        match (
            op.inferred_param_types.first().unwrap(),
            op.function_name.as_ref(),
        ) {
            //u256 operations
            (Some(YulType::U256), "add" | "mul" | "sub" | "and" | "or" | "xor") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U256);
                return;
            }

            (Some(YulType::U256), "iszero" | "eq" | "lt") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);
                return;
            }
            (Some(YulType::U256), "shl" | "shr") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(1);
                self.add_unknown(YulType::U256);
                return;
            }

            //other operations
            (
                Some(YulType::U32) | None,
                "add" | "sub" | "mul" | "div" | "gt" | "lt" | "eq" | "and" | "or",
            ) => {
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);
                self.add_line(op.function_name.as_ref());
                return;
            }

            //iszero
            (Some(YulType::U32) | None, "iszero") => {
                self.add_line("push.0");
                self.add_line("eq");
                self._consume_top_stack_values(1);
                self.add_unknown(YulType::U32);
                return;
            }

            _ => {
                todo!()
            }
        };
    }

    //Transpile an if statement
    //Branches are created to preserve the state of the stack before and after the if statement.
    //See the Branch struct for more details on this
    fn transpile_if_statement(&mut self, op: &ExprIfStatement) {
        self.transpile_op(&op.first_expr);
        self._consume_top_stack_values(1);
        self.add_line("if.true");
        self.indent();
        self.begin_branch();
        self.transpile_block(&op.second_expr);
        self.end_branch();
        self.outdent();
        self.add_line("end");
    }

    //Transpile a literal
    //If the value is u32, a single push will occur
    //If the value is u256, 8 elements will be pushed onto the stack. See push_u256 for more details
    fn transpile_literal(&mut self, literal: &ExprLiteral) {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U256) {
                    self.push_u256(*value);
                } else {
                    self.push(*value);
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    //FIXME: Still needs comments
    //see push identifier to top
    fn transpile_variable_reference(&mut self, op: &ExprVariableReference) {
        let typed_identifier = self.get_typed_identifier(&op.identifier);
        self.push_identifier_to_top(typed_identifier.clone());
    }

    //TODO: stack management not quite working
    //Transpiles a function declaration
    //First, the transpiler gets the stack values that the function will push onto the Miden stack and the function parameters
    //are added to the transpiler's scoped_identifiers. The function is compiled into a procedure and the block is transpiled.
    //After transpiling the function into a Miden procedure, the function is added to user functions with the output stack state.
    //The transpiler stack is reset after transpiling the function declaration and the scoped parameters are removed.
    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        self.stack = Stack(
            op.params
                .iter()
                .map(|param| StackValue::from(param))
                .collect(),
        );
        for param in &op.params {
            self.scoped_identifiers
                .insert(param.identifier.clone(), param.clone());
        }
        // let stack_target = self.stack.clone();
        self.add_line(&format!("proc.{}", op.function_name));
        self.indent();
        self.transpile_block(&op.block);
        // self.target_stack(stack_target);
        for return_ident in &op.returns {
            self.push_identifier_to_top(return_ident.clone());
        }
        self.drop_after(op.returns.len());
        let function_stack = self.stack.clone();
        self.stack = Stack::default();
        self.outdent();
        self.add_line("end");
        self.user_functions
            .insert(op.function_name.clone(), function_stack);
        for param in &op.params {
            self.scoped_identifiers.remove(&param.identifier);
        }
    }

    //TODO: update placeholder
    fn transpile_break(&mut self) {}

    //TODO: update placeholder
    fn transpile_leave(&mut self) {}

    //TODO: update placeholder
    fn transpile_continue(&mut self) {}

    //TODO: update placeholder
    fn transpile_default(&mut self, op: &ExprDefault) {}

    //FIXME: Still needs comments
    //just transpile each case block
    //TODO: update placeholder
    fn transpile_case(
        &mut self,
        op: &ExprCase,
        switch: &ExprSwitch,
        switch_var: TypedIdentifier,
        matched_var: TypedIdentifier,
    ) {
        self.begin_accepting_overflow();
        self.transpile_literal(&op.literal);
        if switch.inferred_type == Some(YulType::U256) {
            self.add_line("exec.u256eq_unsafe");
        } else {
            self.add_line("eq");
        }
        self._consume_top_stack_values(2);
        self.stop_accepting_overflow();

        self.begin_branch();
        self.add_line("if.true");
        self.indent();
        self.transpile_assignment(&ExprAssignment {
            identifiers: vec![matched_var.identifier],
            inferred_types: vec![Some(YulType::U32)],
            rhs: Box::new(Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: Some(YulType::U32),
                value: U256::from(1 as u32),
            }))),
        });
        self.transpile_block(&op.block);
        self.end_branch();
        self.outdent();
        self.add_line("end");
    }

    fn add_line(&mut self, line: &str) {
        self.program = format!(
            "{}\n{}{}",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            line
        )
    }

    //Add a new empty line into the Miden assembly
    fn newline(&mut self) {
        self.program = format!("{}\n", self.program)
    }

    //Add a comment in to the Miden assembly. Comments are generated throughout transpilation.
    fn add_comment(&mut self, comment: &str) {
        dbg!(comment);
        self.program = format!(
            "{}\n{}# {} #",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            comment
        )
    }

    // TODO: re-order AST to have all functions first
    //Function to transpile expressions into Miden instructions
    //See the transpilation function for each expression for more detail on each case
    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.transpile_literal(value),
            Expr::Assignment(op) => self.transpile_assignment(op),
            Expr::DeclareVariable(op) => self.transpile_variable_declaration(op),
            Expr::ForLoop(op) => self.transpile_for_loop(op),
            Expr::Variable(op) => self.transpile_variable_reference(op),
            Expr::Block(op) => self.transpile_block(op),
            Expr::IfStatement(op) => self.transpile_if_statement(op),
            Expr::FunctionCall(op) => self.transpile_miden_function(op),
            Expr::Repeat(op) => self.transpile_repeat(op),
            // We've already compiled the functions
            Expr::FunctionDefinition(op) => (),
            Expr::Break => self.transpile_break(),
            Expr::Continue => self.transpile_continue(),
            Expr::Leave => self.transpile_leave(),
            Expr::Switch(op) => self.transpile_switch(op),
            _ => unreachable!(),
        }
    }

    //Adds all procedures defined in the u256.masm file as utility functions that can be called in the transpiled Miden program
    //Ex. u256add_unsafe, u256sub_unsafe
    fn add_utility_functions(&mut self) {
        let bytes = include_bytes!("./miden_asm/u256.masm");
        let procs = String::from_utf8(bytes.to_vec()).unwrap();
        self.add_line(&format!("{}\n", procs));
    }
}

//Transpile a Miden program from a Vec of expressions and return the compiled Miden program as a string
pub fn transpile_program(expressions: Vec<Expr>) -> String {
    //Initalize the transpiler
    let mut transpiler = Transpiler {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 0,
        stack: Stack::default(),
        scoped_identifiers: HashMap::new(),
        program: "".to_string(),
        user_functions: HashMap::default(),
        branches: VecDeque::new(),
        accept_overflow: false,
    };
    //optimize the abstract syntax tree
    let ast = optimize_ast(expressions);
    //add utility functions from the u256.masm file
    transpiler.add_utility_functions();
    transpiler.add_line("# end std lib #");

    //transpile function declarations first so that they can be called in the Miden program
    for expr in &ast {
        match expr {
            Expr::FunctionDefinition(op) => transpiler.transpile_function_declaration(&op),
            _ => (),
        }
    }

    //start the Miden program
    transpiler.add_line("begin");
    transpiler.indent();
    //transpile each expression in the abstract syntax tree
    for expr in ast {
        transpiler.transpile_op(&expr);
    }
    transpiler.outdent();
    //end the Miden program
    transpiler.add_line("end");
    //return the Miden program as a string
    transpiler.program
}
