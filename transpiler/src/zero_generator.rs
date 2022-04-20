use std::{collections::HashMap, ops::Deref};

use crate::types::*;

use primitive_types::U256;
use zero_machine_code::instructions::*;

//Struct that enables transpilation management into System Zero instructions by keeping track of the stack frames.
struct Transpiler {
    instructions: Vec<GeneralInstruction>,
    stack_frame: StackFrame,
    label_count: usize,
    variable_count: usize,

    current_for_loop: Vec<(String, String, String)>,
    /* variables: HashMap<TypedIdentifier, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
    branches: VecDeque<Branch>,
    accept_overflow: bool,
    memory_offset: u64,
    procs_used: HashSet<String>, */
}

const LOCAL_VARS_START_ADDRESS: u32 = 0;
const EVALUATION_ADDRESS: u32 = 1 << 12;
// const FIRST_OPERAND_ADDRESS: u32 = EVALUATION_ADDRESS + 8;
// const MAX_NUM_OPERANDS: u32 = 12;
// const MAX_OPERAND_ADDRESS: u32 = FIRST_OPERAND_ADDRESS + (MAX_NUM_OPERANDS + 1) * 8;

type Scope = HashMap<String, (YulType, u32)>;

#[derive(Clone)]
struct StackFrame {
    current_offset: u32,
    scopes: Vec<Scope>,
}

impl StackFrame {
    fn new() -> Self {
        StackFrame {
            current_offset: LOCAL_VARS_START_ADDRESS,
            scopes: Vec::new(),
        }
    }

    fn current_scope(&mut self) -> Scope {
        self.scopes[self.scopes.len() - 1].clone()
    }

    fn add_scope(&mut self, scope: Scope) {
        self.scopes.push(scope)
    }

    fn remove_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_variable(&mut self, name: &String) -> Option<(YulType, u32)> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return Some(scope.get(name).unwrap().clone());
            }
        }

        None
    }
}

impl Transpiler {
    fn new() -> Self {
        Transpiler {
            instructions: Vec::new(),
            stack_frame: StackFrame::new(),
            label_count: 0,
            variable_count: 0,
            current_for_loop: Vec::new(),
        }
    }

    fn add_instruction(&mut self, inst: GeneralInstruction) {
        self.instructions.push(inst);
    }

    fn add_real_instruction(&mut self, inst: Instruction) {
        self.add_instruction(GeneralInstruction::Real(inst))
    }

    fn add_label(&mut self, label: String) {
        let label_inst = PseudoInstruction::Label { label };
        self.add_instruction(GeneralInstruction::Pseudo(label_inst));
    }

    fn add_jump(&mut self, addr: ImmediateOrMacro) {
        let jump_inst = PseudoInstruction::Jump { addr };
        self.add_instruction(GeneralInstruction::Pseudo(jump_inst));
    }

    fn add_jump_if(&mut self, cond: LocalOrImmediate, addr: ImmediateOrMacro) {
        let jump_inst = PseudoInstruction::JumpIf { cond, addr };
        self.add_instruction(GeneralInstruction::Pseudo(jump_inst));
    }

    fn add_initialize(&mut self, name: String) {
        let init = PseudoInstruction::Init { name };
        self.add_instruction(GeneralInstruction::Pseudo(init));
    }

    fn add_increment(&mut self, x: LocalOrImmediate) {
        let incr = PseudoInstruction::Incr { x };
        self.add_instruction(GeneralInstruction::Pseudo(incr));
    }

    fn new_if_label(&mut self) -> String {
        let lbl = format!("if{}", self.label_count);
        self.label_count += 1;
        lbl
    }

    fn new_loop_label(&mut self) -> (String, String, String) {
        let pre = format!("pre{}", self.label_count);
        let after_block = format!("after{}", self.label_count);
        let post = format!("post{}", self.label_count);
        self.label_count += 1;
        (pre, after_block, post)
    }

    fn new_variable(&mut self) -> String {
        let name = format!("var{}", self.variable_count);
        self.variable_count += 1;
        name
    }

    // Postcondition: if `expr` is a true Yul "Expression" (a function call, literal, or variable reference),
    // its evaluation is left at EVALUATION_ADDRESS
    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.transpile_literal(value),
            Expr::FunctionDefinition(op) => self.transpile_function_declaration(op),
            Expr::FunctionCall(op) => self.transpile_function_call(op),
            Expr::IfStatement(op) => self.transpile_if_statement(op),
            Expr::Assignment(op) => self.transpile_assignment(op),
            Expr::DeclareVariable(op) => self.transpile_variable_declaration(op),
            Expr::ForLoop(op) => self.transpile_for_loop(op),
            Expr::Block(op) => self.transpile_block(op),
            Expr::Switch(op) => self.transpile_switch(op),
            Expr::Variable(op) => self.transpile_variable_reference(op),
            Expr::Repeat(op) => self.transpile_repeat(op),
            Expr::Break => self.transpile_break(),
            Expr::Continue => self.transpile_continue(),
            // Expr::Leave => self.transpile_leave(),
            _ => unreachable!(),
        }
    }

    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        let function_name = op.function_name.clone();
        let end_label = format!("end_of_{}", function_name);

        self.add_jump(ImmediateOrMacro::AddrOf(end_label.clone()));
        self.add_label(function_name);

        // CALLING CONVENTION

        for expr in op.block.exprs.clone() {
            self.transpile_op(&expr);
        }

        self.add_real_instruction(Instruction::Ret);
        self.add_label(end_label);
    }

    // Postcondition: the function's return value is left at EVALUATION_ADDRESS
    fn transpile_function_call(&mut self, op: &ExprFunctionCall) {
        let function_name = op.function_name.clone();

        let mut index = 0;
        for expr in op.exprs.iter() {
            self.transpile_op(expr);
            self.add_real_instruction(Instruction::CalleeWrite {
                index: index,
                val: LocalOrImmediate::Local(EVALUATION_ADDRESS),
            });
            index += 1; // TODO: handle u256's
        }

        // TODO: more than one return value
        let return_types = op.inferred_return_types.clone();
        assert!(return_types.len() < 2);

        self.add_real_instruction(Instruction::Call {
            addr: ImmediateOrMacro::AddrOf(function_name),
        });

        if return_types.len() == 1 {
            let location_to_read_to = EVALUATION_ADDRESS;

            self.add_real_instruction(Instruction::CalleeWrite {
                index: 0,
                val: LocalOrImmediate::Local(location_to_read_to),
            });
        }
    }

    // Postcondition: the variable's value is left at EVALUATION_ADDRESS
    fn transpile_variable_reference(&mut self, var: &ExprVariableReference) {
        let (ty, address) = self.stack_frame.get_variable(&var.identifier).unwrap();
        
        let move_inst = match ty {
            YulType::U32 => Instruction::Move32 {
                val: LocalOrImmediate::Local(address),
                dst: EVALUATION_ADDRESS,
            },
            YulType::U256 => Instruction::Move256 {
                val: address,
                dst: EVALUATION_ADDRESS,
            },
        };
        self.add_real_instruction(move_inst);
    }

    // Postcondition: the function's return value is left at EVALUATION_ADDRESS
    fn transpile_literal(&mut self, literal: &ExprLiteral) {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U32) {
                    self.place_u32_on_stack((*value).try_into().unwrap());
                } else {
                    self.place_u256_on_stack(*value);
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn transpile_if_statement(&mut self, op: &ExprIfStatement) {
        self.transpile_op(&op.first_expr);
        let prop = EVALUATION_ADDRESS;
        let dest = self.new_if_label();
        self.add_jump_if(
            LocalOrImmediate::Local(prop),
            ImmediateOrMacro::AddrOf(dest.clone()),
        );
        self.transpile_block(&op.second_expr);
        self.add_label(dest);
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let identifier = &op.identifiers[0];

        let scope = self.stack_frame.current_scope();
        let offset = scope.get(identifier).unwrap();

        self.transpile_op(op.rhs.deref());
        let rhs = EVALUATION_ADDRESS;

        // TODO: deal with U256 case
        let move_inst = Instruction::Move32 {
            val: LocalOrImmediate::Local(rhs),
            dst: offset.1,
        };
        self.add_real_instruction(move_inst);
    }

    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        // TODO: more than one identifier in variable declaration
        assert_eq!(op.typed_identifiers.len(), 1);
        let identifier = &op.typed_identifiers[0];

        let scope = self.stack_frame.current_scope();
        let offset = scope.get(&identifier.identifier).unwrap();

        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs.deref());
            let rhs = EVALUATION_ADDRESS;

            // TODO: deal with U256 case
            let move_inst = Instruction::Move32 {
                val: LocalOrImmediate::Local(rhs),
                dst: offset.1,
            };
            self.add_real_instruction(move_inst);
        }
    }

    fn transpile_for_loop(&mut self, op: &ExprForLoop) {
        self.transpile_block(&op.init_block);
        let (pre, after_block, post) = self.new_loop_label();

        self.add_label(pre.clone());
        self.transpile_op(&op.conditional);

        let prop = EVALUATION_ADDRESS;
        self.add_jump_if(
            LocalOrImmediate::Local(prop), 
            ImmediateOrMacro::AddrOf(post.clone()),
        );

        self.current_for_loop.push((pre.clone(), after_block.clone(), post.clone()));
        self.transpile_block(&op.interior_block);
        self.current_for_loop.pop();

        self.add_label(after_block.clone());
        self.transpile_block(&op.after_block);
        self.add_jump(ImmediateOrMacro::AddrOf(pre));

        self.add_label(post);
    }

    fn transpile_break(&mut self) {
        let (_pre, _after_block, post) = self.current_for_loop.last().unwrap().clone();
        self.add_jump(ImmediateOrMacro::AddrOf(post));
    }

    fn transpile_continue(&mut self) {
        let (_pre, after_block, _post) = self.current_for_loop.last().unwrap().clone();
        self.add_jump(ImmediateOrMacro::AddrOf(after_block));
    }

    fn transpile_switch(&mut self, op: &ExprSwitch) {

    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        self.scan_block_for_variables(op);

        for op in &op.exprs {
            self.transpile_op(op);
        }

        self.end_block_scope();
    }

    fn transpile_repeat(&mut self, op: &ExprRepeat) {
        let (pre, after_block, post) = self.new_loop_label();
        self.add_label(pre.clone());
        
        let counter = self.new_variable();
        self.add_initialize(counter.clone());

        let jump = Instruction::JumpGE {
            x: LocalOrImmediate::Immediate(ImmediateOrMacro::AddrOf(counter.clone())),
            y: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(op.iterations)),
            addr: ImmediateOrMacro::AddrOf(post.clone()),
        };
        self.add_real_instruction(jump);

        self.current_for_loop.push((pre.clone(), after_block.clone(), post.clone()));
        self.transpile_block(&op.interior_block);
        self.current_for_loop.pop();

        self.add_label(after_block.clone());
        self.add_increment(LocalOrImmediate::Immediate(ImmediateOrMacro::AddrOf(counter.clone())));

        self.add_jump(ImmediateOrMacro::AddrOf(pre));
        self.add_label(post);
    }

    fn scan_block_for_variables(&mut self, block: &ExprBlock) {
        let mut current_offset = self.stack_frame.current_offset;
        let mut new_scope: HashMap<String, (YulType, u32)> = HashMap::new();

        for expr in block.exprs.clone() {
            match expr {
                Expr::DeclareVariable(e) => {
                    for t in e.typed_identifiers {
                        let iden = t.identifier;
                        let typ = t.yul_type;
                        new_scope.insert(iden, (typ, current_offset));
                        current_offset += match typ {
                            YulType::U32 => 1,
                            YulType::U256 => 8,
                        };
                    }
                }
                _ => {}
            }
        }

        self.stack_frame
            .add_scope(new_scope);
    }

    fn end_block_scope(&mut self) {
        self.stack_frame.remove_scope();
    }

    fn place_u32_on_stack(&mut self, val: u32) {
        let destination = EVALUATION_ADDRESS;

        let move_inst = Instruction::Move32 {
            val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(val)),
            dst: destination,
        };
        self.add_real_instruction(move_inst);
    }

    fn place_u256_on_stack(&mut self, val: U256) {
        let mut cur_location = EVALUATION_ADDRESS;
        let mut cur_val = val;

        for _ in 0..8 {
            let cur: u32 = (cur_val % (1u64 << 32)).try_into().unwrap();
            let move_inst = Instruction::Move32 {
                val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(cur)),
                dst: cur_location,
            };
            self.add_real_instruction(move_inst);
            cur_location += 1;

            cur_val = cur_val / (1u64 << 32);
        }
    }
}

//Return a list of Generalized System Zero instructions from a Vec of Yul expressions
pub fn transpile_program(expressions: Vec<Expr>) -> Vec<GeneralInstruction> {
    let mut transpiler = Transpiler::new();
    for expression in &expressions {
        transpiler.transpile_op(expression);
    }
    transpiler.instructions
}
