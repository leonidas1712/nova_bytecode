use crate::ops::*;
use crate::err::Result;
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

    pub fn run(&self)->Result<()> {
        let first=self.chunk.get_op(self.ip)?;
        
        Ok(())
    }
}