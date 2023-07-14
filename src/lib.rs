pub mod ops;
pub mod vm;
pub mod err;
pub mod stack;

use ops::*;

pub fn run() {
    
}

#[test]
fn test_stack_ops() {
    let mut c2=Chunk::new();

    let idx=c2.add_constant(Value::Number(2), 1);
    let idx2=c2.add_constant(Value::Number(3), 1);
    let idx3=c2.add_constant(Value::Number(5), 1);

    c2.write_op(Inst::OpConstant(idx), 1);
    c2.write_op(Inst::OpConstant(idx2), 1);
    c2.write_op(Inst::OpAdd, 1); // 2+3

    c2.write_op(Inst::OpConstant(idx3), 1);
    c2.write_op(Inst::OpDiv, 1); // 5/5 = 1

    c2.write_op(Inst::OpNegate, 1); // -1

    c2.write_op(Inst::OpConstant(idx3), 1);
    c2.write_op(Inst::OpMul, 1); // -1*5=-5
    

    c2.write_op(Inst::OpReturn, 1);

    let mut vm=vm::VM::new(c2);
    let res=vm.run().unwrap();

    assert_eq!(res.to_string(), "-5");
}