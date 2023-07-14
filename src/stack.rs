use crate::ops::{*};
use crate::err::*;
use std::fmt::Display;
use std::vec;

pub trait Stack<T> {
    fn push(&mut self,val: T)->Result<()>;

    fn pop(&mut self)->Result<T>;

    fn peek(&self) -> &Option<T>;

    fn clear(&mut self);

    fn is_empty(&self) -> bool;
}

impl<T:Display + Copy> Display for FixedStack<T> {
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

// needs to be const so stack size known at compile time
// Fixed size value stack - less overhead than Vec

const STACK_SIZE:usize=2000;
pub struct FixedStack<T:Copy> {
    stack:[Option<T>; STACK_SIZE],
    stack_top:usize // the next place to slot
}

impl<T:Copy> FixedStack <T> {
    pub fn new()-> FixedStack<T> {
        let stack:[Option<T>;STACK_SIZE]=[None;STACK_SIZE];
        FixedStack {
            stack,
            stack_top:0
        }
    }
}

impl<T:Copy> Stack<T> for FixedStack <T> {
    fn push(&mut self,val: T)->Result<()>{
        if self.stack_top > STACK_SIZE {
            return errn!("Maximum stack size {} exceeded: stack overflow", STACK_SIZE);
        }

        self.stack[self.stack_top] = Some(val);
        self.stack_top += 1;
        Ok(())
    }

    fn pop(&mut self)->Result<T>{
        let stack_top=self.stack_top;
        if stack_top == 0 {
           return errn!("Pop from empty stack");
        } 

        let res=self.stack.get(stack_top - 1).unwrap();
        self.stack_top -= 1;

        match res {
            Some(val) => Ok(*val),
            None => errn!("Popped None from stack")
        }
    }

    fn peek(&self) -> &Option<T> {
        if self.is_empty() {
            &None
        } else {
        self.stack.get(self.stack_top-1).unwrap()
        }
    }

    fn clear(&mut self) {
        self.stack_top=0;
    }

    fn is_empty(&self) -> bool {
        self.stack_top == 0
    }
}

pub struct VecStack<T> {
    stack:Vec<T>,
    cap:usize
}

impl <T> VecStack<T> {
    pub fn new(cap:usize)->VecStack<T> {
        VecStack { stack: vec![], cap }
    }
}

impl <T> VecStack<T> {
    fn push(&mut self,val: T)->Result<()>{
        if self.stack.len() > self.cap {
            return errn!("Maximum stack size {} exceeded: stack overflow", self.cap);
        }

        self.stack.push(val);
        Ok(())
    }

    fn pop(&mut self)->Result<T>{
        self.stack.pop().ok_or(errn_i!("Pop from empty stack"))
    }

    // different from fixed stack
    fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    fn clear(&mut self) {
        self.stack.clear()
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[test]
fn test_vec_stack() {
    let mut st:VecStack<&str>=VecStack::new(20);
    st.push("hi");
    st.push("hello");
    st.push("3rd");

    assert_eq!(&"3rd", st.peek().unwrap());
    st.pop();

    assert_eq!(&"hello", st.peek().unwrap());
    st.push("3rd");

    let mut v:Vec<&str>=vec![];

    while !st.is_empty() {
        v.push(st.pop().unwrap());
    }

    assert_eq!(v, vec!["3rd", "hello", "hi"]);
    assert!(st.pop().is_err());
    assert!(st.peek().is_none());

    for _ in 0..21 {
        st.push("hi");
    }

    assert!(st.push("hi").is_err());

    st.clear();
    assert!(st.is_empty());
}


#[test]
fn test_stack() {
    let mut st=FixedStack::new();
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
