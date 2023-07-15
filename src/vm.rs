
use std::rc::{Rc};

use crate::data::{ops::*, stack::*};
use crate::utils::err::*;
use crate::data::ops::Inst::*;

// new(chunk), execute()->Result

/*
    fn interpret(&mut self, source:&str) {

    }
*/

const VAL_STACK_MAX:usize=2000;
pub struct VM {
    chunk:Chunk, // 'c: lifetime of Chunk
    ip:usize, // index of next op to execute,
    value_stack:VecStack<Value> // this should have same layout as Compiler.locals,
    // call_stack: VecStack<CallFrame<'function>>
        // call frame refers to function potentially on value stack
}

impl VM {
    pub fn new()->VM {

        VM {
            chunk:Chunk::new(),
            ip:0,
            value_stack:VecStack::new(VAL_STACK_MAX)
        }
    }

    // get current instruction and increase ip
    pub fn get_curr_inst(&self)->Option<&Inst> {
        let curr=self.chunk.get_op(self.ip);
        curr
    }

    fn reset(&mut self) {
        self.chunk=Chunk::new();
        self.ip=0;
        self.value_stack.clear();
    }

    // &'c mut VM<'c> - the excl. ref must live as long as the object => we can't take any other refs once the 
        // ref is created
    // &mut VM<'c> -> an exclusive ref to VM that has its own lifetime
    pub fn run(&mut self, chunk:Chunk)->Result<Value> {
        self.reset();
        self.chunk=chunk;

        macro_rules! bin_op {
            ($op:tt) => {
                {
                    let stack=&mut self.value_stack;
                    let right=stack.pop()?.expect_int()?;
                    let left=stack.pop()?.expect_int()?;
                    stack.push(Value::num(left $op right))?;
                }
            };
        }

        loop {
            let curr=self.get_curr_inst();
            if curr.is_none() {
                break Ok(Value::Number(1)) // exit code 1
            }  

            let curr=curr.unwrap();

            match curr { 
                // print top of stack and break   
                OpReturn => {
                    let res=self.value_stack.pop()?;
                    break Ok(res);
                },
                // get constant at idx in chunk, push onto stack
                OpConstant(idx) => {
                    let i=*idx;

                    let ct=self.chunk.get_constant(i);
                    
                    let get:Result<Value>=ct
                        .ok_or(errc_i!("Invalid index for constant:{}", i));

                    let get=get?;
                    self.value_stack.push(get)?;
                },
                OpNegate => {
                    let stack=&mut self.value_stack;
                    let top=stack.pop()?.expect_int()?;
                    stack.push(Value::num(top*-1))?;
                },
                OpAdd => bin_op!(+),
                OpSub => bin_op!(-),   
                OpMul => bin_op!(*),
                OpDiv => bin_op!(/),    
                // concat last two
                OpConcat => {
                    let stack=&mut self.value_stack;

                    let right=stack.pop()?;
                    let right=right.expect_string()?;
                    
                    let left=stack.pop()?;
                    let left=left.expect_string()?.clone();

                    let new_val=Value::ObjString(left+right);
                    stack.push(new_val)?;
                }     
            }

            // advance ip - may cause issue since ip advanced before match (unavoidable)
            self.ip+=1;
        }
    }
}