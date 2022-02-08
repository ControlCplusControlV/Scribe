use std::collections::HashMap;

use crate::types::*;

struct Context {
    variables: HashMap<String, u32>,
    indentation: u32,
    next_open_memory_address: u32,
}

fn declare_var(program: &mut String, op: &ExprDeclareVariable, context: &mut Context) {
    let address = context.next_open_memory_address;
    context.next_open_memory_address += 1;
    context.variables.insert(op.identifier.clone(), address);
    if let Some(rhs) = &op.rhs {
        transpile_op(&rhs, program, context);
        add_line(program, "padw", context);
        add_line(program, "drop", context);
        add_line(program, &format!("mem.pop.{}", address), context);
    }
}

fn assignment(program: &mut String, op: &ExprAssignment, context: &mut Context) {
    let address = context.variables.get(&op.identifier).unwrap().clone();
    transpile_op(&op.rhs, program, context);
    add_line(program, "padw", context);
    add_line(program, "drop", context);
    add_line(program, &format!("mem.pop.{}", address), context);
}

fn block(program: &mut String, op: &ExprBlock, context: &mut Context) {
    for op in &op.exprs {
        transpile_op(&op, program, context);
    }
}

fn for_loop(program: &mut String, op: &ExprForLoop, context: &mut Context) {
    block(program, &op.init_block, context);
    transpile_op(&op.conditional, program, context);
    add_line(program, &format!("while.true"), context);
    context.indentation += 4;
    block(program, &op.interior_block, context);
    block(program, &op.after_block, context);
    transpile_op(&op.conditional, program, context);
    context.indentation -= 4;
    add_line(program, &format!("end"), context);
}

fn add(program: &mut String, op: &ExprFunctionCall, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("add"), context);
}

fn mul(program: &mut String, op: &ExprFunctionCall, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("mul"), context);
}

fn gt(program: &mut String, op: &ExprFunctionCall, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("gt"), context);
}

fn if_statement(program: &mut String, op: &ExprIfStatement, context: &mut Context) {
    transpile_op(&op.first_expr, program, context);
    add_line(program, &format!("if.true"), context);
    context.indentation += 4;
    block(program, &op.second_expr, context);
    context.indentation -= 4;
    add_line(program, &format!("end"), context);
}

fn lt(program: &mut String, op: &ExprFunctionCall, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("lt"), context);
}

fn insert_literal(program: &mut String, value: u128, context: &mut Context) {
    add_line(program, &format!("push.{}", value), context);
}

fn load_variable(program: &mut String, op: &ExprVariableReference, context: &mut Context) {
    let address = context.variables.get(&op.identifier).unwrap();
    add_line(program, &format!("mem.push.{}", address), context);
    add_line(program, "dup", context);
    add_line(program, "dropw", context);
}

fn add_line(program: &mut String, line: &str, context: &Context) {
    *program = format!(
        "{}\n{}{}",
        program,
        " ".repeat(context.indentation.try_into().unwrap()),
        line
    )
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
            } else if (op.function_name == "mul") {
                mul(program, op, context)
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

// TESTS

// #[ignore]
// #[test]
// fn test_add_compilation() {
//     let mut program = "begin\npush.0\npush.0\npush.0".to_string();
//     let ops = vec![
//         Expr::DeclareVariable(ExprDeclareVariable {
//             identifier: "foo".to_string(),
//             rhs: Some(Box::new(Expr::Literal(12))),
//         }),
//         Expr::DeclareVariable(ExprDeclareVariable {
//             identifier: "bar".to_string(),
//             rhs: Some(Box::new(Expr::Literal(15))),
//         }),
//         Expr::FunctionCall(ExprFunctionCall {
//             function_name: "add".to_string(),
//             first_expr: Box::new(Expr::Variable(ExprVariableReference {
//                 identifier: "foo".to_string(),
//             })),
//             second_expr: Box::new(Expr::Variable(ExprVariableReference {
//                 identifier: "bar".to_string(),
//             })),
//         }),
//     ];
//     let mut context = Context {
//         variables: HashMap::new(),
//         next_open_memory_address: 1,
//     };
//
//     for op in ops {
//         transpile_op(&op, &mut program, &mut context);
//     }
//     add_line(&mut program, "end");
//
//     println!("{}", program);
//     assert_eq!(
//         program,
//         "begin
// push.0
// push.0
// push.0
// push.12
// mem.store.0
// push.15
// mem.store.1
// mem.load.0
// mem.load.1
// add
// end"
//     );
// }
