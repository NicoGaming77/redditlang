use std::collections::HashMap;

use self::compile_node::Compile;
use crate::{
    bug,
    parser::{Node, Tree},
};
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module,
    values::PointerValue,
};

pub mod compile_node;
pub mod linking;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
}

#[derive(Clone)]
pub struct Scope<'a> {
    pub variables: HashMap<String, PointerValue<'a>>,
}

pub struct CompileMetadata<'a> {
    pub basic_block: BasicBlock<'a>,
    pub function_scope: Scope<'a>,
}

pub fn compile<'a>(compiler: &Compiler<'a>, tree: &Tree, compile_meta: &mut CompileMetadata<'a>) {
    for node in tree {
        compile_one(&compiler, &node, compile_meta);
    }
}

pub fn compile_one<'a>(
    compiler: &Compiler<'a>,
    node: &Node,
    compile_meta: &mut CompileMetadata<'a>,
) {
    match node {
        Node::Loop(r#loop) => r#loop.compile(compiler, compile_meta),
        Node::Break(r#break) => r#break.compile(compiler, compile_meta), // Need to fix,                                                   but won't                                          it's hard
        Node::Function(_) => todo!(),
        Node::Call(call) => call.compile(compiler, compile_meta),
        Node::Throw(_) => todo!(),
        Node::Import(_) => todo!(),
        Node::Module(_) => todo!(),
        Node::TryCatch(_) => todo!(),
        Node::Variable(var) => var.compile(compiler, compile_meta),
        Node::Assignment(_) => todo!(),
        Node::If(r#if) => r#if.compile(compiler, compile_meta),
        Node::Class(_) => todo!(),
        Node::Return(_) => todo!(),
        Node::Expr(_) => bug!("EXPR_IS_STATEMENT_COMPILER"),
    }
}
