use std::collections::HashMap;

use crate::data::{ops::*, stack::*};
use crate::parser::parser::*;
use crate::utils::err::*;
use crate::data::ops::Inst::*;
use crate::utils::misc::calc_hash;

const VAL_STACK_MAX:usize=2000;

// may not need to store chunk

// variables: stores hash of string name -> Value
// get: use hash of string name to get value
// set: set hash -> value
// get name using hash:?
pub struct VM {
    ip:usize, // index of next op to execute,
    value_stack:VecStack<Value>, // this should have same layout as Compiler.locals,
    globals:HashMap<u64,Value> // store u64 hash -> value instead
    // call_stack: VecStack<CallFrame<'function>> 
        // call frame refers to function potentially on value stack
}

// VM: runtime (compilation ends with the chunk)

impl VM {
    pub fn new()->VM {
        VM {
            ip:0,
            value_stack:VecStack::new(VAL_STACK_MAX),
            globals:HashMap::new()
        }
    }

    fn reset(&mut self) {
        self.ip=0;
        self.value_stack.clear();
    }

    fn add_global(&mut self, hash:u64, value:Value) {
        self.globals.insert(hash, value);
    }

    /// Get global value given string name
    pub fn get_global_value<K>(&self, name:K)->Option<&Value> where K:ToString{
        let hash=calc_hash(name.to_string());
        self.globals.get(&hash)
    }

    /// Always returns err variant
    fn err(&self, line:usize, msg:&str)->Result<()> {
        // (RuntimeError) [line 1] Error at end - Expected a token
        let line=format!("[line {}]", line);

        let msg=format!("{} {}", line, msg.to_string());
        errn!(msg)
    }

    // &'c mut VM<'c> - the excl. ref must live as long as the object => we can't take any other refs once the 
        // ref is created
    // &mut VM<'c> -> an exclusive ref to VM that has its own lifetime
    pub fn run(&mut self, chunk:Chunk, reset:bool)->Result<Value> {
        if reset {
            self.reset();
        }

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

        log::debug!("Chunk at start:{}", chunk);

        loop {
            // let curr=self.get_curr_inst(&chunk);
            let curr=chunk.get_op(self.ip);
            if curr.is_none() {
                break Ok(Value::Unit) // exit code 1
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

                    let ct=chunk.get_constant(i);
                    
                    let get:Result<Value>=ct
                        .ok_or(errn_i!("Invalid index for constant:{}", i));

                    let get=get?;
                    self.value_stack.push(get)?;
                },
                OpNegate => {
                    let stack=&mut self.value_stack;
                    let top=stack.pop()?.expect_int()?;
                    stack.push(Value::num(top*-1))?;
                },
                OpAdd =>  {
                    let stack=&mut self.value_stack;
                    let right=stack.pop()?;
                    let left=stack.pop()?;

                    if left.expect_int().is_ok() {
                        let left=left.expect_int()?;
                        let right=right.expect_int()?;
                        stack.push(Value::num(left + right))?;
                    } else if left.expect_string().is_ok() {
                        let left=left.expect_string()?;
                        let right=right.expect_string()?;
                        let left=left.to_owned();
                        let res=left+right;
                        stack.push(Value::ObjString(res))?;
                    } else {
                        let msg=format!("Expected number or string but got: {}", left.to_string());
                        return errn!(msg);
                    }
                },
                OpSub => bin_op!(-),   
                OpMul => bin_op!(*),
                OpDiv => bin_op!(/),   
                OpSetGlobal(hash) => {
                    log::debug!("OpSet");
                    log::debug!("{:?}", self.value_stack);        

                    let value=self.value_stack.pop()?;
                    self.add_global(*hash, value);

                    log::debug!("Set:{:?}",self.globals);
                },
                // idx of identifier in constants
                OpGetGlobal(hash) => {
                    log::debug!("Get {:?} {:?} idx:{}", self.globals, chunk, hash);
                    let value=self.globals.get(hash); // could add line num to value

                    match value {
                        Some(val) => {
                            self.value_stack.push(val.to_owned())?;
                        },
                        None => {

                            // use string interning in chunk to store hash->string for strings
                            let msg=format!("Variable is not defined.");
                            self.err(1, &msg)?;
                        }
                    }
    
                }
            }

            // advance ip - may cause issue since ip advanced before match (unavoidable)
            self.ip+=1;
        }
    }

    /// false: don't reset for run
    pub fn interpret_with_reset(&mut self, source:&str, reset:bool)->Result<Value>{
        let mut chunk=Chunk::new();
        let mut parser=Parser::new(source);

        parser.compile(&mut chunk)?;

        // let chunk=compile(source)?; // turn source into bytecode, consts etc
        self.run(chunk, reset)
    }

    pub fn interpret(&mut self, source:&str)->Result<Value>{
        self.interpret_with_reset(source, true)
    }
}