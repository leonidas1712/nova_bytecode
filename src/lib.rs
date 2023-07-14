pub mod ops;
pub mod vm;
pub mod err;
pub mod stack;

use ops::*;

pub fn run() {
    let mut c2=Chunk::new();
    let i=c2.add_constant(Value::num(500), 1);
    c2.write_op(Inst::OpConstant(i), 1);

    let i=c2.add_constant(Value::num(700), 1);
    c2.write_op(Inst::OpConstant(i), 2);

    let mut string=String::from("this is a string");

    let i=c2.add_constant(Value::ObjString(&string), 1);
    c2.write_op(Inst::OpConstant(i), 2);
    
    let mut vm=vm::VM::new(c2);

    println!("{:?}", vm.run());


    
    let mut st:[Option<Value>;5]=[None;5];
    st[0]=Some(Value::num(10));
    st[1]=Some(Value::num(20));

    println!("{:?}", st);
}