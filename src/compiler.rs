use std::process::id;

use crate::data::stack::VecStack;
use crate::scanner::tokens::Token;
use crate::utils::err::*;
use crate::data::ops::*;
use crate::data::stack::STACK_SIZE;
use crate::utils::misc::calc_hash;

// scanner: makes tokens
// parser: uses tokens and sets current/previous
// emit

// locals, scopeDepth
// Local: Token, depth

// chunk being written to (tied to Function) + locals which have tokens<'src>
#[derive(Debug)]
pub struct Compiler {
    locals:Vec<Local>,
    curr_depth:usize
}   

impl<'src> Compiler {
    pub fn new()->Compiler {
        Compiler { locals: Vec::with_capacity(STACK_SIZE), curr_depth: 0 }
    }

    pub fn begin_scope(&mut self) {
        self.curr_depth+=1;
    }

    pub fn end_scope(&mut self)->usize {
        if !self.is_local() {
            assert!(self.locals.is_empty());
            return 0;
        }
        
        self.curr_depth-=1;

        let curr=self.curr_depth;
        let mut count:usize=0;
        // pop vars from curr scope
        loop {
            match self.locals.last() {
                Some(loc) => {
                    if loc.depth==curr {
                        break;
                    }
                    self.locals.pop();
                    count += 1;
                },
                None => {
                    break;
                }
            }
        }

        count
    }

    pub fn is_local(&self)->bool {
        self.curr_depth > 0
    }

    /// If local found, return corresponding index in value stack to resolve
    pub fn resolve_local(&self, token:&str)->Option<usize> {
        let n=self.locals.len();
        for (idx,loc) in self.locals.iter().rev().enumerate() {
            if loc.is_equal_to(&token) {
                return Some(n-1-idx)
            }
        }
        None
    }

    /// Only add local if curr scope is local. Return idx of local if it was added.
    pub fn add_local(&mut self, token:&str)->Option<usize>{
        if self.is_local() {
            let local=Local::new(token, self.curr_depth);
            self.locals.push(local);
            Some(self.locals.len()-1)
        } else {
            None
        }
    }
}

/// Local variable by its identifier
#[derive(Debug)]
pub struct Local {
    token_hash:u64, // hash identifier e.g hash("x")
    depth:usize
}

impl<'src> Local {
    pub fn new(ident:&str, depth:usize)->Local {
        Local { token_hash: calc_hash(ident), depth }
    }

    // compare hash to hash of input
    pub fn is_equal_to(&self, other:&str)->bool {
        let other_h=calc_hash(other);
        self.token_hash==other_h
    }
}

// pub fn compile(source:&str)->Result<Chunk> {
//     println!("Compiling:{source}");
//     let mut first=source.char_indices();
//     let _get_first=first.next();
//     Ok(Chunk::new())
// }

// Compiler struct: 
    // chunk to write to: first has a pointer to the Chunk being written to
    // later: has a pointer to the function or closure
        // then the function/closure has the chunk