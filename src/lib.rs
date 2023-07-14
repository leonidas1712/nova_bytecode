pub mod ops;
pub mod vm;
pub mod err;
pub mod stack;

use ops::*;

pub fn run() {
    let mut c2=Chunk::new();

    let idx=c2.add_constant(Value::Number(2), 1);
    let idx2=c2.add_constant(Value::Number(30), 1);

    c2.write_op(Inst::OpConstant(idx2), 1);
    c2.write_op(Inst::OpConstant(idx), 1);
    c2.write_op(Inst::OpSub, 1);

    c2.write_op(Inst::OpConstant(idx2), 1);
    c2.write_op(Inst::OpAdd, 1);

    c2.write_op(Inst::OpConstant(idx), 1);
    c2.write_op(Inst::OpDiv, 1);

    c2.write_op(Inst::OpConstant(idx2), 1);
    c2.write_op(Inst::OpMul, 1);

    c2.write_op(Inst::OpReturn, 1);

    let mut vm=vm::VM::new(c2);

    println!("{:?}", vm.run());
    // vm.run();

    // println!("{:?}", st);
}