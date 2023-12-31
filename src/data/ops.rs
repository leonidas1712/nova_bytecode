use std::{fmt::{Display}, vec, collections::{HashMap, hash_map::DefaultHasher}, hash::Hasher};
use std::hash::Hash;
use crate::{utils::{err::*, misc::StringIntern}, vm::{self, VM}};

// Inst, Chunk, Value

// type BinOp=for<'v> fn(&'v Value<'v>,&'v Value<'v>)->Value<'v>;
// type UnaryOp=fn(&mut Value);


// no point hashing opget/set because it needs to be hashed later/stored
#[derive(Debug)]
// Instruction
// binaryop: takes two args from stack, applies op, pushes onto stack
pub enum Inst {
    OpReturn,
    OpConstant(usize), // idx in const pool, -> load idx onto stack
    OpGetGlobal(String), 
    OpSetGlobal(String),
    OpGetLocal(usize),
    OpSetLocal(usize), // idx into value stack
    OpLoadString(u64),
    OpIfFalseJump(usize), // jump to idx if cond is false
    OpJump(usize), // unconditional jump when branch is taken
    OpPrint,
    OpNegate,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpPop,
    OpEndScope(usize,bool), // num to pop, is_expr,
    OpTrue,
    OpFalse,
    OpNot
}

impl<'src> Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type IntType=isize;

// idea: ValueStack might need to be dynamic so it can own values
// but CallStack with CallFrames might not need to be
    // CallStack: owns CallFrame (T=CallFrame)
        // CallFrame: ip(usize), FunctionPtr(&Function), slots(usize)
        // slots is first index in ValueStack that this callframe can use


// Alt Obj repr:
     /*
        struct Object {
           enum ObjType
    
        }
    
        enum ObjType {
            String(Rc<String>),
            Function(Rc<Fn>)
        }
    */

// chunk: write_string -> write string to chunk constants and return hash
  // only add entry if DNE
// get_string(hash) -> use hash to get string

// Observation: Rc may only be needed for Function (?)
    // Function is referred to in callframe as well but other values may only be on val stack?

// store Ident(String,line) so we can retrieve for err
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
// 16 bytes - could benefit from using &Value in stack so that everything is 8 bytes (ptr size)
// page size: 16kb on M1 Mac -> 1024 values fit in one page
// M1 Mac TLB=32MB -> 2^11 pages = 2048 pages
// 2048*1024 ~ 2M values can fit in the TLB - the stack can be size 2M and still benefit from TLB cache locality

// CPU cache size on M1: 128kb => 8192 values while still benefiting from cache locality

pub enum Value {
    Number(IntType),
    Bool(bool),
    ObjString(u64), // change to use u64 -> Copy (hash of string in VM)
    Unit // empty type
}

impl Value {
    pub fn num(n:IntType)->Value  {
        Self::Number(n)
    }

    pub fn expect_int(&self)->Result<IntType> {
        match self {
            Self::Number(n) => Ok(*n),
            Self::Bool(b) => Ok(if *b { 1 } else { 0 }),
            Self::ObjString(_) => err_other!("Expected number but got a string"),
            _ => err_other!("Expected number but got: '{}'", self.to_string())
        }
    }

    pub fn expect_string(&self)->Result<u64> {
        match self {
            Self::ObjString(hash) => Ok(*hash),
            _ => err_other!("Expected string but got: '{}'", self.to_string())
        }
    }

    pub fn expect_bool(&self)->Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            Self::Number(n) => Ok(!n.eq(&0)),
            Self::ObjString(_) => err_other!("Expected bool but got a string"),
            _ => err_other!("Expected bool but got: {}", self.to_string())
        }
    }

    pub fn is_unit(&self)->bool {
        match self {
            Self::Unit => true,
            _ => false
        }
    }

    pub fn get_hash(&self)->u64 {
        // let mut hasher:Box<dyn Hasher>=Box::new(DefaultHasher::new());
        let mut s=DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Display for Value  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr=match &self {
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::ObjString(s) => format!("\"{}\"", s.to_string()),
            Self::Unit => String::from("()")
        };

        write!(f, "{}", repr)
    }
}

#[derive(Debug)]
struct LineEncoding(usize,usize); // (line number, occurences)
#[derive(Debug)]
struct Lines {
    lines:Vec<LineEncoding>
}

impl Lines {
    pub fn new()->Lines {
        Lines {
            lines:vec![]
        }
    }

    pub fn add_line(&mut self, line_num:usize) {
        let last=self.lines.last_mut();

        match last {
            Some(le) if le.0.eq(&line_num) =>  le.1 += 1,
            _ => self.lines.push(LineEncoding(line_num, 1))
        }
    }

    // index if we uncompress e.g  (12,2), (14,3), (12,1)..
    pub fn get_line(&self, idx:usize)->Option<usize> {
        // idx < start+occurences => end
        let mut start=0;
        for le in self.lines.iter() {
            let (line,occur)=(le.0,le.1);
            if idx < start + occur {
                return Some(line)
            }

            start+=occur;
        }
        None
    }
}
// represents a series of bytecode instructions along with context
// need to return an index: to benefit from cache locality instead of associating vals with insts direct
// Value->usize (for checking if val exists)
#[derive(Debug)]
pub struct Chunk {
    ops:Vec<Inst>, // ops stack: order matters
    constants:Vec<Value>, // pool of constants - order doesnt matter
    constants_map:HashMap<u64,usize>, // val.hash->idx stored in constants
    op_lines:Lines, // line numbers
    constant_lines:Lines, // two arrs because index goes along with the enum (less confusing),
    pub strings:StringIntern
}

impl<'src> Chunk {
    pub fn new()->Self {
        Chunk {
            ops:vec![], constants:vec![], op_lines:Lines::new(), constant_lines:Lines::new(), constants_map:HashMap::new(),
            strings:StringIntern::new()
        }
    }

    /// len-1 of ops if not empty
    pub fn get_ip(&self)->Option<usize> {
        if self.ops.is_empty() {
            None
        } else {
            Some(self.ops.len()-1)
        }
    }

     // easier to use idx even though slightly more overhead
        // we have no easy way to get the next Inst unlike in C besides idx
    pub fn get_op(&self, idx:usize)->Option<&Inst> {
        self.ops.get(idx)
    }

    pub fn get_op_mut(&mut self, idx:usize)->Option<&mut Inst> {
        self.ops.get_mut(idx)
    }

    /// return index where op was written
    pub fn write_op(&mut self, op:Inst, line:usize)->usize {
        self.ops.push(op);
        self.op_lines.add_line(line);
        self.ops.len()-1
    }

    pub fn get_constant(&self, idx:usize)->Option<Value> {
        self.constants.get(idx).map(|v| v.to_owned())
    }

    /// Get index of value given hash. Returns none if hash DNE
    fn get_idx(&self, hash:u64)->Option<&usize> {
        self.constants_map.get(&hash)
    }

    /// Returns index where constant was added - for use in OP_CONSTANT
    /// Adds to constants pool
    pub fn add_constant(&mut self, value:Value, line:usize)->usize {
        let val_hash=value.get_hash();
        let val_idx=self.get_idx(val_hash);

        match val_idx {
            // Exists: return existing index
            Some(idx) => {
                *idx
            },
            None => {
                let constants=&mut self.constants;
                constants.push(value);

                self.constant_lines.add_line(line);

                // add hash to map
                let idx=constants.len()-1; 
                self.constants_map.insert(val_hash, idx);

                idx
            }
        }
    }

    /// add const + add OP_CONSTANT
    pub fn write_constant(&mut self, value:Value, line:usize) {
        let idx=self.add_constant(value, line);
        self.write_op(Inst::OpConstant(idx), line);
    }

    /// Adds string and emits OpLoadString with hash
    pub fn load_string(&mut self, string:String, line:usize) {
        let hash=self.strings.add_string(string);
        let op=Inst::OpLoadString(hash);
        self.write_op(op, line);
    }

    pub fn get_line_of_constant(&self, idx:usize) -> Option<usize>{
        self.constant_lines.get_line(idx)
    }

    pub fn get_line_of_op(&self, idx:usize)->Option<usize> {
        self.op_lines.get_line(idx)
    }
}

impl<'src> Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut code=String::from("Ops:\n");
        for (idx,op) in self.ops.iter().enumerate() {
            
            let fmt=match op {
                Inst::OpConstant(i) => {
                    let c=self.get_constant(*i).unwrap();
                    let const_string=format!("{}", c.to_string().as_str());

                    format!("OpConstant (line {}) | {}\n", self.op_lines.get_line(idx).unwrap(), const_string)
                }
                _ => format!("{} (line {})\n", op.to_string().as_str(), self.op_lines.get_line(idx).unwrap())

            };
            code.push_str(fmt.as_str());
        }
        code.push_str("\n\nUnique constants:\n");

        for (idx,c) in self.constants.iter().enumerate() {
            let fmt=format!("{} (line {})\n", c.to_string().as_str(), self.constant_lines.get_line(idx).unwrap());
            code.push_str(fmt.as_str());
        }

        write!(f, "{}", code)
    }
}

#[test]
fn test_lines() {
    let mut lines=Lines::new();
    assert_eq!(None, lines.get_line(0));

    lines.add_line(12);
    lines.add_line(12);

    lines.add_line(14);
    lines.add_line(14);
    lines.add_line(14);

    lines.add_line(15);

    assert_eq!(Some(12), lines.get_line(0));
    assert_eq!(Some(14), lines.get_line(2));
    assert_eq!(Some(14), lines.get_line(4));
    assert_eq!(Some(15),lines.get_line(5));
    assert_eq!(None, lines.get_line(6));

}