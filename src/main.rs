pub mod ops;
use ops::*;

fn main() {
    let mut c=Chunk::new();

    let idx=c.add_constant(Value::num(10),123);
    let idx2=c.add_constant(Value::num(20),123);

    c.write_chunk(Code::OpConstant(idx),123);
    c.write_chunk(Code::OpConstant(idx),123);
    c.write_chunk(Code::OpConstant(idx),123);


    c.write_chunk(Code::OpConstant(idx2),124);
    c.write_chunk(Code::OpConstant(idx),123);

    c.write_chunk(Code::OpReturn,125);
    
    println!("{}", c);
}
