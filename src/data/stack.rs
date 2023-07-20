
use crate::utils::err::*;
use std::fmt::{Display, Debug, format};
use std::vec;

pub trait Stack<T> {
    fn push(&mut self,val: T)->Result<()>;

    fn pop(&mut self)->Result<T>;

    fn peek(&self) -> &Option<T>;

    fn clear(&mut self);

    fn is_empty(&self) -> bool;
}

impl<T:Display + Copy + Debug> Display for FixedStack<T> {
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

pub const STACK_SIZE:usize=2000;

pub struct FixedStack<T:Copy + Debug> {
    stack:[Option<T>; STACK_SIZE],
    stack_top:usize // the next place to slot
}

impl<T:Copy + Debug> Debug for FixedStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v:Vec<String>=vec![];
        for i in 0..self.stack_top {
            let fmt=format!("{:?}", self.stack[i]);
            v.push(fmt);
        }

        let str=v.join(",");
        write!(f, "FixedStack: [{}]", str)
    }
}

impl<T:Copy + Debug> FixedStack <T> {
    pub fn new()-> FixedStack<T> {
        let stack:[Option<T>;STACK_SIZE]=[None;STACK_SIZE];
        FixedStack {
            stack,
            stack_top:0
        }
    }

    /// None if value at the index is none or idx is out of bounds
    pub fn get(&self, idx:usize)->Option<T> {
        if idx < STACK_SIZE {
            self.stack[idx]
        } else {
            None
        }
    }

    /// Panics if idx is invalid
    pub fn set(&mut self, idx:usize, item:T) {
        self.stack[idx]=Some(item);
    }
}

impl<T:Copy + Debug> Stack<T> for FixedStack <T> {
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

// indirection to enforce capacity
#[derive(Debug)]
pub struct VecStack<T> {
    pub stack:Vec<T>,
    cap:usize
}

impl <T> VecStack<T> {
    pub fn new(cap:usize)->VecStack<T> {
        VecStack { stack: vec![], cap }
    }
}

impl <T> VecStack<T> {
    pub fn push(&mut self,val: T)->Result<()>{
        if self.stack.len() > self.cap {
            return errn!("Maximum stack size {} exceeded: stack overflow", self.cap);
        }

        self.stack.push(val);
        Ok(())
    }

    pub fn pop(&mut self)->Result<T>{
        self.stack.pop().ok_or(errn_i!("Pop from empty stack"))
    }

    // different from fixed stack
    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn clear(&mut self) {
        self.stack.clear()
    }

    pub fn get(&self, idx:usize)->Option<&T>{
        self.stack.get(idx)
    }

    pub fn set(&mut self, idx:usize, item:T) {
        // self.stack.set
        self.stack[1]=item;
    }

    pub fn is_empty(&self) -> bool {
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

    let mut st:VecStack<String>=VecStack::new(20);
    st.push(String::from("hi")); // can do non copy
}


#[test]
fn test_stack() {
    let mut st:FixedStack<usize>=FixedStack::new();

    st.push(10);
    st.push(20);

    let third=st.push(30);
    assert!(third.is_ok());
    assert_eq!("[10,20,30]",st.to_string());

    while !st.is_empty() {
        let p=st.pop().unwrap();
        println!("Pop:{}", p);
    }

    st.push(30);
    st.push(40);
    st.pop();
    st.push(50); // [30,50]

    let p=st.pop().unwrap();
    assert_eq!(p.to_string(), "50");

    let pk=st.peek().unwrap(); // [30]
    assert_eq!("30", pk.to_string());

    st.pop();

    let pk=st.peek();
    assert!(pk.is_none());
    assert!(st.is_empty());
}
