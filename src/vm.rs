use anyhow::__private::kind::AdhocKind;

use crate::{errc, errn_i};
use crate::ops::{*, Inst::*};
use crate::err::*;

// new(chunk), execute()->Result
pub struct VM {
    chunk:Chunk,
    ip:usize // index of next op to execute
}

impl VM {
    pub fn new(chunk:Chunk)->Self {
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
                    let get:Result<&Value>=self.chunk.get_constant(i).ok_or(errc_i!("Invalid index for constant:{}", i));
                    let get=get?;               
                    println!("{}", get);
                    
                     // let get=self.chunk.get_constant(i);
                
                    // if let Some(c) = get {
                    //     println!("{}", c.to_string());
                    // } else {
                    //     return errc!("Invalid constant index: {}",i);
                    // }
                }
            }
        }

        Ok(())
    }
}