use std::collections::{HashMap, HashSet, VecDeque};

use primitive_types::U256;

use crate::{ast_optimization::optimize_ast, types::*, utils::convert_u256_to_pushes};

/// Struct to keep track of which utility functions should be added 
/// This will become crucial later for run based optimization (inline vs turn it into a proc determining factor)
struct IncludedProcs {
    u256add_unsafe:bool,
    u256and_unsafe:bool,
    u256div_by_one:bool,
    u256eq_unsafe:bool,
    u256gt_unsafe:bool,
    u256iszero_unsafe:bool,
    u256lt_unsafe:bool,
    u256mul_unsafe:bool,
    u256or_unsafe:bool,
    u256shl_unsafe:bool,
    u256shr_unsafe:bool,
    u256sub_unsafe:bool,
    u256xor_unsafe:bool,
}

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
    memory_offset: u64,
    procs_used: IncludedProcs,
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

//FIXME: Add a note about how the transpiler variables, stack ect do not cross over with miden. They are just to keep track of
//what the miden stack/memory will look like so we essentially need to states, one for miden and one for the transpiler
impl Transpiler {
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
            self.move_identifier_to_top(
                v.typed_identifier
                    .clone()
                    .expect("Need to deal w/ this case"),
                false,
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

    //Generally we keep all stack values accessible (aka, only use the first 16 slots). This will
    //temporarily disable that protection, for cases where we know we're going to be consuming the
    //values on top of the stack, before other values need to be accessed
    fn begin_accepting_overflow(&mut self) {
        self.accept_overflow = true;
    }

    //Re-enable our overflow protection (saving values that go below the 16 accesible slots, to
    //memory)
    fn stop_accepting_overflow(&mut self) {
        self.accept_overflow = false;
    }

    //Add an unkown value onto our stack, for example if we evaluate `add(1,2)`, we know there's a
    //new value on the stack but it isn't assigned to a variable yet. This is often followed by a
    //top_is_var
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
        self.move_identifier_to_top(typed_identifier, false);
        self.pop_top_stack_value_to_memory(None);
    }

    //Fetches an identifier from either memory or the stack, and moves it to the top. Will dup from
    //the stack if dup is true, otherwise will move up
    fn move_identifier_to_top(&mut self, typed_identifier: TypedIdentifier, dup: bool) {
        let mut offset = 0;
        let mut index = 0;
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
                index += 1;
                return false;
            })
            .cloned();
        match stack_value {
            Some(stack_value) => {
                self.add_comment(&format!(
                    "pushing {} to the top",
                    typed_identifier.identifier
                ));
                self.indent();
                if dup {
                    self.stack.0.insert(
                        0,
                        StackValue {
                            typed_identifier: None,
                            yul_type: typed_identifier.yul_type,
                        },
                    );
                    self.dup_from_offset(offset, stack_value.yul_type);
                } else {
                    let sv = self.stack.0.remove(index);
                    self.stack
                        .0
                        .insert(0, StackValue::from(&typed_identifier.clone()));
                    self.move_from_offset(offset, stack_value.yul_type);
                }
                self.outdent()
            }

            //If it cant find the identifier in the stack it will load it from memory
            None => {
                self.load_identifier_from_memory(typed_identifier);
            }
        }
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
    //If the element is a u32, it takes up one 32bit element of one memory address. When the
    //element is a u256, it takes up two addresses completely.
    fn push_from_memory_to_top_of_stack(&mut self, address: u32, yul_type: &YulType) {
        match yul_type {
            YulType::U32 => {
                self.add_line(&format!("push.mem.{}", address));
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
    //and depending on the type it may have to dup 8 elements
    fn dup_identifier(&mut self, typed_identifier: TypedIdentifier) {
        self.move_identifier_to_top(typed_identifier, true);
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

    //Function to push a u32 value on both the miden stack and our stack
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

    //Function to check the stack size and determine whether or not the push will overflow into
    //inaccesible stack slots (17+). If it would push values past the 16th slot, those values are
    //saved into memory. See comments on accept_overflow for more details on when the
    //transpiler uses memory vs the stack
    fn prepare_for_stack_values(&mut self, yul_type: &YulType) {
        if self.accept_overflow {
            return;
        }
        while self.get_size_of_stack() + yul_type.miden_stack_width() > 16 {
            self.add_comment(&format!(
                "stack would be too large after {}, popping to memory",
                yul_type,
            ));
            let bottom_stack_value = self.stack.0.last().unwrap();
            if bottom_stack_value.typed_identifier.is_none() {
                break;
            }
            self.indent();
            self.pop_bottom_var_to_memory();
            self.outdent();
        }
    }

    //Function to remove the bottom variable from the stack and move it to memory.
    //Under the hood, this function moves the bottom value to the top of the stack and then saves it to memory.
    //See pop_top_stack_value_to_memory for more details.
    fn pop_bottom_var_to_memory(&mut self) {
        let stack_value = self.stack.0.last().cloned().unwrap();

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
            "Moving {} to top of stack, {} values above",
            stack_value.typed_identifier.as_ref().unwrap().identifier,
            num_stack_values_above
        ));
        self.move_from_offset(num_stack_values_above, stack_value.yul_type);
        self.stack.0.pop();
        self.stack.0.insert(0, stack_value.clone());
        self.pop_top_stack_value_to_memory(None);
    }

    fn move_from_offset(&mut self, offset: u32, yul_type: YulType) {
        match yul_type {
            YulType::U32 => match offset {
                0 => {}
                1 => {
                    self.add_line(&format!("swap"));
                }
                n => {
                    self.add_line(&format!("movup.{}", n));
                }
            },
            YulType::U256 => {
                match offset {
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
    }

    //Function to remove the top stack value from the stack and save it into memory.
    //If the variable is already stored in memory, the transpiler will update the value. Else, the
    //transpiler will get the next available memory address. If the value is a u256,
    //mem.pop.address and mem.pop.address+1 can be used to pop two words into memory
    fn pop_top_stack_value_to_memory(&mut self, address: Option<u32>) -> u32 {
        let stack_value = self.stack.0.first().unwrap().clone();

        let address = address.unwrap_or(
            match stack_value
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
            },
        );
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
                self.add_line(&format!("pop.mem.{}", address));
            }
            YulType::U256 => {
                self.add_line(&format!("popw.mem.{}", address));
                self.add_line(&format!("popw.mem.{}", address + 1));
            }
        }
        self.newline();
        address
    }

    //Return the size of the stack, accounting for u256 values taking up 8 values.
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

    //Consume n stack values from the top of our stack. This doesn't affect miden's stack.
    fn _consume_top_stack_values(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
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
    fn drop_after_returns(&mut self, returns: Vec<TypedIdentifier>) {
        let mut address = 0;
        for _ in &returns {
            self.pop_top_stack_value_to_memory(Some(address));
            address += 2;
        }
        let mut miden_size = self.get_size_of_stack();
        while miden_size > 0 {
            if miden_size > 3 {
                self.add_line("dropw");
                miden_size -= 4;
            } else {
                self.add_line("drop");
                miden_size -= 1;
            }
        }
        self.stack.0 = vec![];
        for function_return in returns {
            address -= 2;
            self.push_from_memory_to_top_of_stack(address, &function_return.yul_type);
        }
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
    //when that function is called, we have to modify our stack by pushing those two numbers to the top of the stack.
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
        for typed_identifier in &op.typed_identifiers {
            self.scoped_identifiers.insert(
                typed_identifier.identifier.clone(),
                typed_identifier.clone(),
            );
        }
        assert_eq!(op.typed_identifiers.len(), 1);
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
    //Note that ExprAssignment has parameters of
    // pub identifiers: Vec<String>, (ie. variable names)
    // pub inferred_types: Vec<Option<YulType>>, (ie. data types)
    // pub rhs: Box<Expr>, (right hand side of :=)
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

        //Transpiles the right hand side expression and pushes the expr to the top
        self.transpile_op(&op.rhs);
        //Assigns the top stack value (right hand side of the expression) to the variable name passed in as typed_identifier
        self.top_is_var(typed_identifier);
        self.outdent();
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
    // Note that ExprSwitch has four parts
    // pub default_case: Option<ExprBlock>,
    // pub inferred_type: Option<YulType>,
    // pub expr: Box<Expr>,
    // pub cases: Vec<ExprCase>,
    fn transpile_switch(&mut self, op: &ExprSwitch) {
        self.add_line("");

        //Define a variable to keep in the transpiler scoped identifiers that represents if the switch expression has been matched
        let transpiler_switch_matched_bool = TypedIdentifier {
            identifier: "switch_matched".to_string(),
            yul_type: YulType::U32,
        };

        //Add the switch_matched variable to the transpiler's known variables
        self.scoped_identifiers.insert(
            transpiler_switch_matched_bool.identifier.clone(),
            transpiler_switch_matched_bool.clone(),
        );

        //Push 0 on the Miden stack. This is the equivalent of pushing the switch_matched variable
        self.add_comment("keeping track of whether we've hit any cases");
        self.prepare_for_stack_values(&YulType::U32);
        self.add_line("push.0");

        //push an unknown on transpiler stack
        self.add_unknown(YulType::U32);
        //declare that top value in stack is the switch_matched variable
        self.top_is_var(transpiler_switch_matched_bool.clone());

        //Transpile the ExprSwitch.expr (expression) and push the expression to the top of the transpiler stack
        // The expression can be any Expr
        //Ex.
        //switch add(x,y)
        //case 10 {x=34}
        //case 11 {x=23}
        //In the above example, ExprSwitch.expr is add(x,y)
        self.transpile_op(&op.expr);

        //Define a transpiler variable that will represent the switch expression Assign ExprSwitch.expr to this variable.
        let transpiler_target_switch_expression = TypedIdentifier {
            identifier: "switch".to_string(),
            yul_type: op.inferred_type.unwrap(),
        };

        //Add the target switch expression to the transpiler's known variables
        self.scoped_identifiers.insert(
            transpiler_target_switch_expression.identifier.clone(),
            transpiler_target_switch_expression.clone(),
        );

        //The value at the top of the stack is the switch expression that was transpiled from transpile_op(&op.expr).
        //Declare the transpiler_target_switch_expression to be this value so that the transpiler knows what value the target switch is.
        self.top_is_var(transpiler_target_switch_expression.clone());

        //For each case in the ExprSwitch.cases in the ExprSwitch passed into transpile_switch()
        for (i, case) in op.cases.iter().enumerate() {
            //Dup the transpiler_target_switch_expression, which results in this value at the top of the stack
            self.dup_identifier(transpiler_target_switch_expression.clone());

            //Now the transpiler stack state is [transpiler_target_switch_expression, transpiler_target_switch_expression]

            //Transpile the case which includes the case literal and case block that will be evaluated if the literal is matched
            //with the targeted switch expression. For more details, check out the transpile_case function.
            self.transpile_case(
                &case,
                &op,
                transpiler_target_switch_expression.clone(),
                transpiler_switch_matched_bool.clone(),
            );
        }

        //If there is a default case, transpile the default case
        if let Some(default_case) = &op.default_case {
            self.add_comment("default case");
            self.dup_identifier(transpiler_switch_matched_bool);
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

    fn transpile_function_args(&mut self, op: &ExprFunctionCall) {
        for expr in op.exprs.clone().into_iter() {
            self.transpile_op(&expr);
        }
    }

    //Transpiles a procedure in miden.
    //For example, if a function call is passed in as an expression, the first parameter type is u256 and the function name is "add"
    //a call to the procedure exec.u256add_unsafe will be added into the program.
    //If the first parameter is u256, the transpiler will use u256 operations
    fn transpile_miden_function(&mut self, op: &ExprFunctionCall) {
        self.add_comment(&format!("{}()", op.function_name));

        if let Some(function_stack) = self.user_functions.clone().get(&op.function_name) {
            self.transpile_function_args(&op);
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
                self.transpile_function_args(&op);
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U256);

                // Next ensure proc is added into the program
                match &op.function_name.as_str() {
                    &"add"=>self.procs_used.u256add_unsafe = true,
                    &"mul"=>self.procs_used.u256mul_unsafe = true,
                    &"sub"=>self.procs_used.u256sub_unsafe = true,
                    &"and"=>self.procs_used.u256and_unsafe = true,
                    &"or"=>self.procs_used.u256or_unsafe = true,
                    &"xor"=>self.procs_used.u256xor_unsafe = true,
                    _ => ()
                }
                return;
            }
            (Some(YulType::U32), "mstore") => {
                let value_expr = op.exprs.get(1).unwrap();
                self.transpile_op(value_expr);
                let address_expr = op.exprs.first().unwrap();
                self.transpile_op(address_expr);
                match value_expr.get_inferred_type().unwrap() {
                    YulType::U32 => {
                        self.add_line(&format!("mul.2 push.{} add", self.memory_offset));
                        self.add_line(&format!("pop.mem"));
                        self._consume_top_stack_values(2);
                    }
                    YulType::U256 => {
                        self.add_line(&format!(
                            "mul.2 push.{} add dup movdn.5",
                            self.memory_offset
                        ));
                        self.add_line(&format!("popw.mem"));
                        self.add_line("add.1 popw.mem");
                        self._consume_top_stack_values(2);
                    }
                };
            }
            (Some(YulType::U32), "mload") => {
                let address_expr = op.exprs.first().unwrap();
                self.transpile_op(address_expr);
                match op.inferred_return_types.first().unwrap().unwrap().clone() {
                    YulType::U32 => {
                        self.add_line(&format!("mul.2 push.{} add", self.memory_offset));
                        self.add_line(&format!("push.mem"));
                        self._consume_top_stack_values(1);
                        self.add_unknown(YulType::U32);
                    }
                    YulType::U256 => {
                        self.add_line(&format!("mul.2 push.{} add dup", self.memory_offset));
                        self.add_line(&format!("add.1 pushw.mem"));
                        self.add_line(&format!("movup.4 pushw.mem"));
                        self._consume_top_stack_values(1);
                        self.add_unknown(YulType::U256);
                    }
                };
            }

            (Some(YulType::U256), "iszero" | "eq" | "lt") => {
                self.transpile_function_args(&op);
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);

                match op.function_name.as_str() {
                    "iszero"=>self.procs_used.u256iszero_unsafe = true,
                    "eq"=>self.procs_used.u256shr_unsafe = true,
                    "lt"=>self.procs_used.u256shr_unsafe = true,
                    "gt"=>self.procs_used.u256gt_unsafe = true,
                }
                return;
            }
            (Some(YulType::U256), "shl" | "shr") => {
                self.transpile_function_args(&op);
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(1);
                self.add_unknown(YulType::U256);

                // Next ensure proc is added into the program
                match op.function_name.as_str() {
                    "shl"=>self.procs_used.u256shl_unsafe = true,
                    "shr"=>self.procs_used.u256shr_unsafe = true,
                }
                return;
            }

            // binary u32 math and boolean ops
            (
                Some(YulType::U32) | None,
                "add" | "sub" | "mul" | "div" | "gt" | "lt" | "eq" | "and" | "or",
            ) => {
                self.transpile_function_args(&op);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);
                self.add_line(op.function_name.as_ref());
                return;
            }

            (Some(YulType::U32) | None, "iszero") => {
                self.transpile_function_args(&op);
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
        self.dup_identifier(typed_identifier.clone());
    }

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
        self.add_line(&format!(
            "proc.{}.{}",
            op.function_name,
            op.params.len() * 2
        ));
        self.indent();
        self.transpile_block(&op.block);
        for return_ident in &op.returns {
            self.dup_identifier(return_ident.clone());
        }
        self.drop_after_returns(op.returns.clone());
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

    //Transpile a case expression
    // Note that ExprCase has two parameters
    // pub literal: ExprLiteral, (this is the case statement)
    // pub block: ExprBlock, (this is the code that is evaluated if the case is matched)
    fn transpile_case(
        &mut self,
        op: &ExprCase,
        switch: &ExprSwitch,
        _transpiler_target_switch_expression: TypedIdentifier,
        transpiler_switch_matched_bool: TypedIdentifier,
    ) {
        //Start to accept overflow, meaning when elements are pushed to the stack making the stack size greater than 16
        // the values will not be saved to memory and the 16th element (at index 15) will be moved out of frame to the 17th element (at index 16)
        self.begin_accepting_overflow();

        //Transpile the case literal and push it to the top of the stack.
        //Ex. case 0 { result := 1 }
        //In the above case, 0 is the case literal and the switch statement will evaluate against the number literal 0
        //Now the transpiler stack state is [case_literal, transpiler_target_switch_expression]
        self.transpile_literal(&op.literal);

        //If the type is u256, use u256eq_unsafe, otherwise use eq to evaluate equality between the case literal
        // and the transpiler_target_switch_expression
        if switch.inferred_type == Some(YulType::U256) {
            self.add_line("exec.u256eq_unsafe");
        } else {
            self.add_line("eq");
        }
        //Tell the transpiler to consume the top stack values, simulating how Miden will consume 2 after eq
        self._consume_top_stack_values(2);

        //Stop accepting overflow, any values that are in the stack that exceed the stack frame are moved to memory.
        self.stop_accepting_overflow();

        //Begin a new branch, for more detail on transpiler branches, see the Branch struct
        self.begin_branch();

        //If the top stack value in Miden is 1 meaning that the eq (or 256eq) was true and the switch was matched
        self.add_line("if.true");
        self.indent();

        //Assign the transpiler_switch_matched_bool the value of 1 in the transpiler and in the in Miden assembly
        self.transpile_assignment(&ExprAssignment {
            identifiers: vec![transpiler_switch_matched_bool.identifier],
            inferred_types: vec![Some(YulType::U32)],
            rhs: Box::new(Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: Some(YulType::U32),
                value: U256::from(1 as u32),
            }))),
        });

        //Transpile the block that will be executed if the case is matched to the switch expression during Miden runtime.
        //All cases blocks are transpiled because we can not know which case is matched during transpilation.
        self.transpile_block(&op.block);

        //End the branch, restoring the transpiler stack state to the state before the case block, but with updated variable values
        self.end_branch();
        self.outdent();
        //End the case in Miden assembly
        self.add_line("end");
    }

    // Adds a line to the miden program output, properly indented
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
        self.program = format!(
            "{}\n{}# {} #",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            comment
        )
    }

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
        memory_offset: 1024,
        procs_used : IncludedProcs {
            u256add_unsafe:false,
            u256and_unsafe:false,
            u256div_by_one: false,
            u256eq_unsafe:false,
            u256gt_unsafe:false,
            u256iszero_unsafe:false,
            u256lt_unsafe:false,
            u256mul_unsafe:false,
            u256or_unsafe:false,
            u256shl_unsafe:false,
            u256shr_unsafe:false,
            u256sub_unsafe:false,
            u256xor_unsafe:false,
        }
    };
    //optimize the abstract syntax tree
    let ast = optimize_ast(expressions);
    //add utility functions from the u256.masm file
    transpiler.add_utility_functions();
    transpiler.add_line("# end std lib #");

    //transpile function declarations first so that the procs are generated before we begin
    //transpiling the body of the miden program
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
