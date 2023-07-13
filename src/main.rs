pub mod ops;
pub mod vm;
pub mod err;

use ops::*;

fn main() {
    let mut c=Chunk::new();

    let idx=c.add_constant(Value::num(10),123);
    let idx2=c.add_constant(Value::num(20),123);

    c.write_op(Inst::OpConstant(idx),123);
    c.write_op(Inst::OpConstant(idx),123);
    c.write_op(Inst::OpConstant(idx),123);


    c.write_op(Inst::OpConstant(idx2),124);
    c.write_op(Inst::OpConstant(idx),123);

    c.write_op(Inst::OpReturn,125);

    let mut c2=Chunk::new();
    let vm=vm::VM::new(c);

}
