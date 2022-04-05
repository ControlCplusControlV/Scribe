use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::types::*;

use primitive_types::U256;
use zero_machine_code::instructions::*;

//Struct that enables transpilation management into System Zero instructions by keeping track of the stack frames.
struct Transpiler {
    instructions: Vec<GeneralInstruction>,
    current_stack_frame: StackFrame,
    previous_stack_frames: Vec<StackFrame>,
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

const EVAL_STACK_START_ADDRESS: u32 = 0;
const LOCAL_VARS_START_ADDRESS: u32 = 1 << 12;

type Scope = HashMap<String, (YulType, u32)>;

struct LocalVariables {
    current_offset: u32,
    scopes: Vec<Scope>,
}

impl LocalVariables {
    fn new() -> Self {
        LocalVariables {
            current_offset: 0,
            scopes: Vec::new(),
        }
    }

    fn current_scope(&mut self) -> Scope {
        self.scopes[self.scopes.len() - 1].clone()
    }

    fn add_scope(&mut self, scope: Scope) {
        self.scopes.push(scope)
    }
}

struct EvaluationStack {
    state: Vec<YulType>,
}

impl EvaluationStack {
    fn new() -> Self {
        EvaluationStack { state: Vec::new() }
    }
}

struct StackFrame {
    local_variables: LocalVariables,
    evaluation_stack: EvaluationStack,
}

impl StackFrame {
    fn new() -> Self {
        StackFrame {
            local_variables: LocalVariables::new(),
            evaluation_stack: EvaluationStack::new(),
        }
    }
}

impl EvaluationStack {
    fn offset_of_ith_element(&mut self, i: usize) -> u32 {
        let mut offset = EVAL_STACK_START_ADDRESS;
        for j in 0..i {
            offset += match self.state[j] {
                YulType::U32 => 1,
                YulType::U256 => 8,
            };
        }
        offset
    }

    fn offset_of_last_element(&mut self) -> u32 {
        self.offset_of_ith_element(self.state.len() - 1)
    }

    fn push(&mut self, typ: YulType) -> LocalOffset {
        self.state.push(typ);
        self.offset_of_last_element()
    }

    fn pop(&mut self) -> LocalOffset {
        let idx = self.offset_of_last_element();
        self.state.pop();
        idx
    }

    fn add3(&mut self) -> Vec<GeneralInstruction> {
        let n = self.state.len();
        assert!(n > 2);

        let x_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n - 2));
        let y_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n - 1));
        let z_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n));

        let dst_addr = self.offset_of_ith_element(n - 2);

        let add3_inst = Instruction::Add3 {
            x: x_addr,
            y: y_addr,
            z: z_addr,
            dst: dst_addr,
        };
        vec![GeneralInstruction::Real(add3_inst)]
    }
}

/*#[derive(Default, Clone)]
struct Branch {
    modified_identifiers: HashSet<TypedIdentifier>,
    stack_before: Stack,
}*/

impl Transpiler {
    fn new() -> Self {
        Transpiler {
            instructions: Vec::new(),
            current_stack_frame: StackFrame::new(),
            previous_stack_frames: Vec::new(),
        }
    }
    fn add_instruction(&mut self, inst: GeneralInstruction) {
        self.instructions.push(inst);
    }

    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        // TODO: calling convention stuff?

        for expr in op.block.exprs.clone() {
            self.transpile_op(&expr);
        }
    }

    fn scan_block_for_variables(&mut self, op: &ExprFunctionDefinition) {
        let mut current_offset = self.current_stack_frame.local_variables.current_offset;
        let mut new_scope: HashMap<String, (YulType, u32)> = HashMap::new();

        for expr in op.block.exprs.clone() {
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

        self.current_stack_frame
            .local_variables
            .add_scope(new_scope);
    }

    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        // TODO: more than one identifier in variable declaration
        assert_eq!(op.typed_identifiers.len(), 1);
        let identifier = &op.typed_identifiers[0];

        let scope = self.current_stack_frame.local_variables.current_scope();
        let offset = scope.get(&identifier.identifier).unwrap();

        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs.deref());
            let rhs = self.current_stack_frame.evaluation_stack.pop();

            // TODO: deal with U256 case
            let move_inst = Instruction::Move32 {
                val: LocalOrImmediate::Local(rhs),
                dst: offset.1,
            };
            self.add_instruction(GeneralInstruction::Real(move_inst));
        }
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let identifier = &op.identifiers[0];

        let scope = self.current_stack_frame.local_variables.current_scope();
        let offset = scope.get(identifier).unwrap();

        self.transpile_op(op.rhs.deref());
        let rhs = self.current_stack_frame.evaluation_stack.pop();

        // TODO: deal with U256 case
        let move_inst = Instruction::Move32 {
            val: LocalOrImmediate::Local(rhs),
            dst: offset.1,
        };
        self.add_instruction(GeneralInstruction::Real(move_inst));
    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        // scan block for variables
        // add new variable mapping for this scope

        for op in &op.exprs {
            self.transpile_op(op);
        }
    }

    fn transpile_literal(&mut self, literal: &ExprLiteral) {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U256) {
                    self.place_u256_on_stack(*value);
                } else {
                    self.place_u32_on_stack((*value).try_into().unwrap());
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn place_u32_on_stack(&mut self, val: u32) -> LocalOffset {
        let location_of_new_space = self.current_stack_frame.evaluation_stack.push(YulType::U32);

        let move_inst = Instruction::Move32 {
            val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(val)),
            dst: location_of_new_space,
        };
        self.add_instruction(GeneralInstruction::Real(move_inst));
        location_of_new_space
    }

    fn place_u256_on_stack(&mut self, val: U256) -> LocalOffset {
        let location_of_new_space = self
            .current_stack_frame
            .evaluation_stack
            .push(YulType::U256);
        let mut cur_location = location_of_new_space;
        let mut cur_val = val;

        for _ in 0..8 {
            let cur: u32 = (cur_val % (1u64 << 32)).try_into().unwrap();
            let move_inst = Instruction::Move32 {
                val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(cur)),
                dst: cur_location,
            };
            self.add_instruction(GeneralInstruction::Real(move_inst));
            cur_location += 1;

            cur_val = cur_val / (1u64 << 32);
        }

        location_of_new_space
    }

    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.transpile_literal(value),
            Expr::Assignment(op) => self.transpile_assignment(op),
            Expr::DeclareVariable(op) => self.transpile_variable_declaration(op),
            // Expr::ForLoop(op) => self.transpile_for_loop(op),
            // Expr::Variable(op) => self.transpile_variable_reference(op),
            Expr::Block(op) => self.transpile_block(op),
            // Expr::IfStatement(op) => self.transpile_if_statement(op),
            Expr::FunctionCall(op) => self.transpile_function_call(op),
            // Expr::Repeat(op) => self.transpile_repeat(op),
            // We've already compiled the functions
            Expr::FunctionDefinition(op) => todo!(),
            // Expr::Break => self.transpile_break(),
            // Expr::Continue => self.transpile_continue(),
            // Expr::Leave => self.transpile_leave(),
            // Expr::Switch(op) => self.transpile_switch(op),
            _ => unreachable!(),
        }
    }

    fn transpile_function_call(&mut self, op: &ExprFunctionCall) {
        // insert CALL
        // add new stack frame
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
