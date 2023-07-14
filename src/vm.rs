
use crate::ops::{*, Inst::*};
use crate::err::*;
use crate::stack::Stack;


// new(chunk), execute()->Result
pub struct VM<'c> {
    chunk:Chunk<'c> , // 'c: lifetime of Chunk
    ip:usize, // index of next op to execute,
    value_stack:Stack<Value<'c>> // vals come from chunk
}

impl<'c>  VM<'c> {
    pub fn new(chunk:Chunk<'c> )->Self {
        VM {
            chunk,
            ip:0,
            value_stack:Stack::new()
        }
    }

    // get current instruction and increase ip
    fn get_curr_inst(&self)->Option<&Inst> {
        let curr=self.chunk.get_op(self.ip);
        // self.ip+=1;
        curr
    }

    pub fn run(&'c mut self)->Result<()> {
        loop {
            let curr=self.get_curr_inst();
            if curr.is_none() {
                break;
            }  

            let curr=curr.unwrap();

            match curr { 
                // print top of stack and break   
                OpReturn => {
                    let res=self.value_stack.pop()?;
                    println!("{res}");
                    break;
                },
                // get constant at idx in chunk, push onto stack
                OpConstant(idx) => {
                    let i=*idx;
                    let get:Result<Value>=self.chunk
                        .get_constant(i)
                        .ok_or(errc_i!("Invalid index for constant:{}", i));

                    let get=get?.to_owned();
                    self.value_stack.push(get)?;
                },
                OpNegate => {
                    let stack=&mut self.value_stack;
                    let top=stack.pop()?.expect_int()?;
                    stack.push(Value::num(top*-1))?;
                }
            }

            // advance ip - may cause issue since ip advanced before match (unavoidable)
            self.ip+=1;
        }

        Ok(())
    }
}