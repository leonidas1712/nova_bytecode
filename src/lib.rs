pub mod ops;
pub mod vm;
pub mod err;
pub mod stack;

use ops::*;

pub fn run() {
    // let mut s1=String::from("hi");
    // let s2=String::from("hello");
    // s1.push_str(&s2);
    // println!("{s2}");
    // println!("{s1}");

    let s1="hi";
    let s2="he";
    let k=s1.to_owned();

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

    println!("{}", c2);
    
    let mut vm=vm::VM::new(c2);

    let vm2=&mut vm;
    let res=vm2.run();

    // vm.get_curr_inst();

    // assert_eq!(res.to_string(), "-5");
    
}