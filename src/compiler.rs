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
    locals:VecStack<Local<'src>>,
    curr_depth:usize
}   

impl<'src> Compiler<'src> {
    pub fn new()->Compiler<'src> {
        Compiler { locals: VecStack::new(STACK_SIZE), curr_depth: 0 }
    }
}

#[derive(Debug)]
pub struct Local<'src> {
    name:Token<'src>,
    depth:usize
}

impl<'src> Local <'src>  {
    pub fn new(token:Token<'src>, depth:usize)->Local<'src> {
        Local { name: token, depth }
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