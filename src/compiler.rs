use crate::utils::err::*;
use crate::data::ops::*;

// scanner: makes tokens
// parser: uses tokens and sets current/previous
// emit

// locals, scopeDepth
// Local: Token, depth

// chunk being written to (tied to Function) + locals which have tokens<'src>

pub struct Compiler {

}   

pub fn compile(source:&str)->Result<Chunk> {
    println!("Compiling:{source}");
    let mut first=source.char_indices();
    let _get_first=first.next();
    Ok(Chunk::new())
}

// Compiler struct: 
    // chunk to write to: first has a pointer to the Chunk being written to
    // later: has a pointer to the function or closure
        // then the function/closure has the chunk