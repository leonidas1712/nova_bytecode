use crate::utils::err::*;
use crate::data::ops::*;

// scanner: makes tokens
// parser: uses tokens and sets current/previous
// emit
pub fn compile(source:&str)->Result<Chunk> {
    println!("Compiling:{source}");
    let mut first=source.char_indices();
    let _get_first=first.next();
    Ok(Chunk::new())
}