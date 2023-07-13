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
    let i=c2.add_constant(Value::num(500), 1);
    c2.write_op(Inst::OpConstant(i), 1);

    let i=c2.add_constant(Value::num(700), 1);
    c2.write_op(Inst::OpConstant(10), 2);
    
    let mut vm=vm::VM::new(c2);
    let res=vm.run().unwrap_err();
    println!("{}", res.to_string());

}
