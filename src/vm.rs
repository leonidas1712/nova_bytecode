use std::collections::HashMap;
use std::hash::Hash;
use std::process::id;

use crate::data::{ops::*, stack::*};
use crate::parser::parser::*;
use crate::utils::err::*;
use crate::data::ops::Inst::*;
use crate::utils::misc::{calc_hash, StringIntern};

const VAL_STACK_MAX:usize=2000;

// may not need to store chunk

// variables: stores hash of string name -> Value
// get: use hash of string name to get value
// set: set hash -> value
// get name using hash:? string intern

// string hash -> string
// Value: ObjString(hash of string)
// comparing strings: just compare hash
// add string (ident or not) => add hash to the map

// need to store ident mapping as well to support printing all vars

// for every string: we want a unique copy of the string, and string cmp should be just cmp hash or ptrs
// 1. given hash, get string
// 2. every string is deduped: same seq of chars -> only one copy

// x="abc"; y="abc"; z="abc"; => should only have one copy and x,y,z refer to same

#[derive(Debug)]
pub struct VM {
    ip:usize, // index of next op to execute,
    value_stack:VecStack<Value>, // this should have same layout as Compiler.locals,
    globals:HashMap<u64,Value>, // store u64 hash -> value instead
    // call_stack: VecStack<CallFrame<'function>> 
        // call frame refers to function potentially on value stack
    strings:StringIntern 
}

// VM: runtime (compilation ends with the chunk)

impl VM {
    pub fn new()->VM {
        VM {
            ip:0,
            value_stack:VecStack::new(VAL_STACK_MAX),
            globals:HashMap::new(),
            strings:StringIntern::new()
        }
    }

    fn reset(&mut self) {
        self.ip=0;
        self.value_stack.clear();
    }

    /// Add global variable given identifier
    fn add_global(&mut self, identifier:String, value:Value) {
        let hash=self.strings.add_string(identifier);
        self.globals.insert(hash, value);
    }

    fn get_global(&self, identifier:&String)->Option<&Value> {
        let hash=calc_hash(identifier);
        self.globals.get(&hash)
    }

    /// Get global value given string name
    pub fn get_global_value<K>(&self, name:K)->Option<&Value> where K:ToString{
        let hash=calc_hash(&name.to_string());
        self.globals.get(&hash)
    }

    /// returns string interned in hash
    fn expect_string(&self, hash:u64)->Result<&String> {
        match self.strings.get_string(hash) {
            Some(sref) => {
                return Ok(sref)
            },
            None => {
                let msg=format!("Invalid hash for string (not interned)");
                self.err(self.ip, &msg)?;
                unreachable!()
            }
        }
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
                // if hash doesnt exist in strings, add loaded str from chunk to strings. else, loadfrom interned
                OpLoadString(hash) => {
                    log::debug!("Load str:{}", hash);
                    let hash=*hash;
                    let has_interned=self.strings.has_string(hash);

                    if !has_interned {
                        let load=chunk.strings.get_string(hash).expect("Invalid string hash from chunk");
                        self.strings.add_string(load.to_string());
                        log::debug!("Loaded str:{}", load);
                    }

                    let obj_str=Value::ObjString(hash);
                    self.value_stack.push(obj_str)?;

                    // match get_interned {
                    //     Some(val) => {

                    //     },
                    //     None => {

                    //     }
                    // }
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
                        let left_hash=left.expect_string()?;
                        let right_hash=right.expect_string()?;

                        let left=self.strings.get_string(left_hash).unwrap();
                        let right=self.strings.get_string(right_hash).unwrap();
                        let left=left.to_owned();
                        let res=left+right;

                        let hash=self.strings.add_string(res);
                        stack.push(Value::ObjString(hash))?;
                    } else {
                        let msg=format!("Expected number or string but got: {}", left.to_string());
                        return errn!(msg);
                    }
                },
                OpSub => bin_op!(-),   
                OpMul => bin_op!(*),
                OpDiv => bin_op!(/),   
                OpSetGlobal(identifier) => {
                    log::debug!("OpSet");
                    log::debug!("{:?}", self.value_stack);        

                    let value=self.value_stack.pop()?;

                    self.add_global(identifier.to_string(), value);

                    log::debug!("Set:{:?}",self.globals);
                },
                // idx of identifier in constants
                OpGetGlobal(ident) => {
                    log::debug!("Get {:?} {:?} idx:{}", self.globals, chunk, ident);
                    let value=self.get_global(ident); // could add line num to value

                    match value {
                        Some(val) => {
                            self.value_stack.push(val.to_owned())?;
                        },
                        None => {

                            // use string interning in chunk to store hash->string for strings
                            let line=chunk.get_line_of_op(self.ip).expect("Invalid index for op line");
                            let msg=format!("Variable '{}' is not defined.", ident);
                            self.err(line, &msg)?;
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
        match self.run(chunk, reset) {
            Ok(val) => Ok(val),
            Err(msg) => {
                let msg=format!("[line {}] {}", self.ip, msg);
                errn!(msg)
            },
        }
    }

    pub fn interpret(&mut self, source:&str)->Result<Value>{
        self.interpret_with_reset(source, true)
    }

    /// Get string representation of value 
    pub fn print_value(&mut self, value:Value)->String {
        match value {
            Value::ObjString(hash) => {
                let load=self.strings.get_string(hash);
                let load=load.expect("Invalid string printed: not found in VM intern");
                format!("\"{}\"",load.to_string())
            },
            _ => value.to_string()
        }
    }

     /// Always returns err variant
     fn err(&self, line:usize, msg:&str)->Result<()> {
        // (RuntimeError) [line 1] Error at end - Expected a token
        let line=format!("[line {}]", line);

        let msg=format!("{} {}", line, msg.to_string());
        err_other!(msg)
    }
}