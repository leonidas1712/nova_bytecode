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
    let res=vm.run().unwrap();

    assert_eq!(res.to_string(), "-5");
    
}

use std::rc::Rc;
/*
    struct Object {
       enum ObjType

    }

    enum ObjType {
        String(Rc<String>),
        Function(Rc<Fn>)
    }
*/
#[test]
fn test_concat() {
    let mut c2=Chunk::new();

    // Value::ValObj(Object::new("hi"))
    // Value::ValObj(Object::new(Function{...}))

    let string1=Rc::new("hi".to_string()); 
    let string2=Rc::new("hello".to_string());

    let idx=c2.add_constant(Value::ObjString(string1), 1);
    let idx2=c2.add_constant(Value::ObjString(string2), 1);

    c2.write_op(Inst::OpConstant(idx), 1);
    c2.write_op(Inst::OpConstant(idx2), 1);
    
    c2.write_op(Inst::OpConcat, 1);
    c2.write_op(Inst::OpReturn, 1);
    
    let mut vm=vm::VM::new(c2);
    let res=vm.run().unwrap();

    assert_eq!(res.to_string(), "hihello");
}