use core::panic;

use crate::{errc, errn_i};
use crate::ops::{*, Inst::*};
use crate::err::*;

// Fixed size value stack - less overhead than Vec
const STACK_SIZE:usize=2000;
pub struct Stack<'val> {
    stack:[Option<Value<'val>>; STACK_SIZE],
    stack_top:usize // the next place to slot
}

impl<'val> Stack <'val> {
    pub fn new()-> Stack<'val> {
        let stack:[Option<Value>;STACK_SIZE]=[None;STACK_SIZE];
        Stack {
            stack,
            stack_top:0
        }
    }

    pub fn push(&mut self,val: Value<'val>)->Result<()>{
        if self.stack_top > STACK_SIZE {
            return errn!("Maximum stack size {} exceeded: stack overflow", STACK_SIZE);
        }

        self.stack[self.stack_top] = Some(val);
        self.stack_top += 1;
        Ok(())
    }

    pub fn pop(&mut self)->Result<Value<'val>>{
        let stack_top=self.stack_top;
        if stack_top == 0 {
           return errn!("Pop from empty value stack");
        } 

        let res=self.stack.get(stack_top - 1).unwrap();
        self.stack_top -= 1;

        match res {
            Some(val) => Ok(val.clone()),
            None => errn!("Popped none value from stack")
        }
    }

    pub fn peek(&self) -> &Option<Value<'val>> {
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

// new(chunk), execute()->Result
pub struct VM<'c> {
    chunk:Chunk<'c> ,
    ip:usize // index of next op to execute
}

impl<'c>  VM<'c> {
    pub fn new(chunk:Chunk<'c> )->Self {
        VM {
            chunk,
            ip:0
        }
    }

    // get current instruction and increase ip
    fn get_curr_inst(&mut self)->Option<&Inst> {
        let curr=self.chunk.get_op(self.ip);
        self.ip+=1;
        curr
    }

    pub fn run(&mut self)->Result<()> {
        loop {
            let curr=self.get_curr_inst();
            if curr.is_none() {
                break;
            }  

            let curr=curr.unwrap();

            match curr {
                OpReturn => break,
                OpConstant(idx) => {
                    let i=*idx;
                    let get:Result<&Value>=self.chunk
                        .get_constant(i)
                        .ok_or(errc_i!("Invalid index for constant:{}", i));

                    let get=get?;               
                    println!("{}", get);
                }
            }
        }

        Ok(())
    }
}

#[test]
fn test_stack() {
    let mut st=Stack::new();
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