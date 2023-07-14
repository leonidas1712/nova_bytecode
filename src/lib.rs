pub mod ops;
pub mod vm;
pub mod err;
pub mod stack;

use ops::*;

pub fn run() {
    let mut c2=Chunk::new();

    let idx=c2.add_constant(Value::Number(50), 1);
    c2.write_op(Inst::OpConstant(idx), 1);

    let idx=c2.add_constant(Value::Number(100), 1);
    c2.write_op(Inst::OpConstant(idx), 1);

    c2.write_op(Inst::OpReturn, 2);

    let mut vm=vm::VM::new(c2);

    println!("{:?}", vm.run());
    // vm.run();

    // println!("{:?}", st);
}