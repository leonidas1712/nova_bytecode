use crate::ops::{*};
use crate::err::*;
use std::fmt::Display;

// Fixed size value stack - less overhead than Vec
const STACK_SIZE:usize=2000;
impl<T:Display + Copy> Display for Stack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items:Vec<String>=vec![];

        for idx in 0..self.stack_top {
            let repr=self.stack.get(idx)
            .and_then(|item| item.map(|t| t.to_string()))
            .unwrap_or(String::from("None"));
            items.push(repr);
        }

        let all=items.join(",");
        write!(f, "[{}]", all)
    }
}
// Stack<T>?
pub struct Stack<T:Copy> {
    stack:[Option<T>; STACK_SIZE],
    stack_top:usize // the next place to slot
}

impl<T:Copy> Stack <T> {
    pub fn new()-> Stack<T> {
        let stack:[Option<T>;STACK_SIZE]=[None;STACK_SIZE];
        Stack {
            stack,
            stack_top:0
        }
    }

    pub fn push(&mut self,val: T)->Result<()>{
        if self.stack_top > STACK_SIZE {
            return errn!("Maximum stack size {} exceeded: stack overflow", STACK_SIZE);
        }

        self.stack[self.stack_top] = Some(val);
        self.stack_top += 1;
        Ok(())
    }

    pub fn pop(&mut self)->Result<T>{
        let stack_top=self.stack_top;
        if stack_top == 0 {
           return errn!("Pop from empty value stack");
        } 

        let res=self.stack.get(stack_top - 1).unwrap();
        self.stack_top -= 1;

        match res {
            Some(val) => Ok(*val),
            None => errn!("Popped none value from stack")
        }
    }

    pub fn peek(&self) -> &Option<T> {
        if self.is_empty() {
            &None
        } else {
            self.stack.get(self.stack_top-1).unwrap()
        }
    }

    pub fn clear(&mut self) {
        self.stack_top=0;
    }

    pub fn is_empty(&self) -> bool {
        self.stack_top == 0
    }
}



#[test]
fn test_stack() {
    let mut st=Stack::new();
    st.push(Value::Number(10));
    st.push(Value::Bool(false));

    let third=st.push(Value::Number(30));
    assert!(third.is_ok());
    assert_eq!("[10,false,30]",st.to_string());

    while !st.is_empty() {
        let p=st.pop().unwrap();
        println!("Pop:{}", p);
    }

    st.push(Value::Number(30));
    st.push(Value::Number(40));
    st.pop();
    st.push(Value::Bool(true)); // [30,true]

    let p=st.pop().unwrap();
    assert_eq!(p.to_string(), "true");

    let pk=st.peek().unwrap(); // [30]
    assert_eq!("30", pk.to_string());

    st.pop();

    let pk=st.peek();
    assert!(pk.is_none());
    assert!(st.is_empty());
}

#[test]
fn test_gen() {
    let mut st=Stack::<Value>::new();
    st.push(Value::Number(10));
    st.push(Value::Bool(false));

    let third=st.push(Value::Number(30));
    assert!(third.is_ok());

    while !st.is_empty() {
        let p=st.pop().unwrap();
        println!("Pop:{}", p);
    }

    st.push(Value::Number(30));
    st.push(Value::Number(40));
    st.pop();
    st.push(Value::Bool(true)); // [30,true]

    let p=st.pop().unwrap();
    assert_eq!(p.to_string(), "true");

    let pk=st.peek().unwrap(); // [30]
    assert_eq!("30", pk.to_string());

    st.pop();

    let pk=st.peek();
    assert!(pk.is_none());
    assert!(st.is_empty());
}