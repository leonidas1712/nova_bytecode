use std::{fmt::{Display, write}, vec};

#[derive(Debug)]
pub enum Code {
    OpReturn,
    OpConstant(usize), // idx in const pool
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum Value {
    Number(usize),
}

impl Value {
    pub fn num(n:usize)->Value {
        Self::Number(n)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr=match &self {
            Self::Number(n) => n.to_string()
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

    // idx 2
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
#[derive(Debug)]
pub struct Chunk {
    ops:Vec<Code>,
    constants:Vec<Value>, // pool of constants
    op_lines:Lines, // line numbers
    constant_lines:Lines // two arrs because index goes along with the enum (less confusing)
}

impl Chunk {
    pub fn new()->Self {
        Chunk {
            ops:vec![], constants:vec![], op_lines:Lines::new(), constant_lines:Lines::new()
        }
    }

    pub fn write_chunk(&mut self, op:Code, line:usize) {
        self.ops.push(op);
        self.op_lines.add_line(line);
    }
    
    // Returns index where constant was added
    pub fn add_constant(&mut self, value:Value, line:usize)->usize {
        let constants=&mut self.constants;
        constants.push(value);

        self.constant_lines.add_line(line);
        constants.len()-1
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut code=String::from("Ops:\n");
        for (idx,op) in self.ops.iter().enumerate() {
            let fmt=format!("{} (line {})\n", op.to_string().as_str(), self.op_lines.get_line(idx).unwrap());
            code.push_str(fmt.as_str());
        }
        code.push_str("\n\nConstants:\n");

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