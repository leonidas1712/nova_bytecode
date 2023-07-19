use crate::data::stack::VecStack;
use crate::scanner::tokens::Token;
use crate::utils::err::*;
use crate::data::ops::*;
use crate::data::stack::STACK_SIZE;

// scanner: makes tokens
// parser: uses tokens and sets current/previous
// emit

// locals, scopeDepth
// Local: Token, depth

// chunk being written to (tied to Function) + locals which have tokens<'src>


    

#[derive(Debug)]
pub struct Compiler<'src> {
    locals:Vec<Local<'src>>,
    curr_depth:usize
}   

impl<'src> Compiler<'src> {
    pub fn new()->Compiler<'src> {
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
    pub fn resolve_local(&self, token:Token<'src>)->Option<usize> {
        let n=self.locals.len();
        for (idx,loc) in self.locals.iter().rev().enumerate() {
            if loc.token.is_equal_by_content(&token) {
                return Some(n-1-idx)
            }
        }
        None
    }

    /// Only add local if curr scope is local. Return idx of local if it was added.
    pub fn add_local(&mut self, token:Token<'src>)->Option<usize>{
        if self.is_local() {
            let local=Local::new(token, self.curr_depth);
            self.locals.push(local);
            Some(self.locals.len()-1)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Local<'src> {
    token:Token<'src>, // identifier e.g "x"
    depth:usize
}

impl<'src> Local <'src>  {
    pub fn new(token:Token<'src>, depth:usize)->Local<'src> {
        Local { token, depth }
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