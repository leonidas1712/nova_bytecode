use crate::ops::{*};
use crate::err::*;

// Fixed size value stack - less overhead than Vec
const STACK_SIZE:usize=2000;
pub struct Stack<'val> {
    stack:[Option<Value<'val>>; STACK_SIZE],
    stack_top:usize // the next place to slot
}

impl<'val> Stack <'val> {
    pub fn new()-> Stack<'val> {
        let stack:[Option<Value>;STACK_SIZE]=[None;STACK_SIZE];
        Stack {
            stack,
            stack_top:0
        }
    }

    pub fn push(&mut self,val: Value<'val>)->Result<()>{
        if self.stack_top > STACK_SIZE {
            return errn!("Maximum stack size {} exceeded: stack overflow", STACK_SIZE);
        }

        self.stack[self.stack_top] = Some(val);
        self.stack_top += 1;
        Ok(())
    }

    pub fn pop(&mut self)->Result<Value<'val>>{
        let stack_top=self.stack_top;
        if stack_top == 0 {
           return errn!("Pop from empty value stack");
        } 

        let res=self.stack.get(stack_top - 1).unwrap();
        self.stack_top -= 1;

        match res {
            Some(val) => Ok(*val),
            None => errn!("Popped none value from stack")
        }
    }

    pub fn peek(&self) -> &Option<Value<'val>> {
        if self.is_empty() {
            &None
        } else {
            self.stack.get(self.stack_top-1).unwrap()
        }
    }

    pub fn clear(&mut self) {
        self.stack_top=0;
    }

    pub fn is_empty(&self) -> bool {
        self.stack_top == 0
    }
}

impl<'val> std::fmt::Display for Stack<'val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items:Vec<String>=vec![];

        for idx in 0..self.stack_top {
            let repr=self.stack.get(idx).unwrap();
            let repr=match repr {
                Some(val) => val.to_string(),
                None => "None".to_string()
            };
            items.push(repr);
        }

        let all=items.join(",");
        write!(f, "[{}]", all)
    }
}


#[test]
fn test_stack() {
    let mut st=Stack::new();
    st.push(Value::Number(10));
    st.push(Value::Bool(false));

    let third=st.push(Value::Number(30));
    assert!(third.is_ok());

    while !st.is_empty() {
        let p=st.pop().unwrap();
        println!("Pop:{}", p);
    }

    st.push(Value::Number(30));
    st.push(Value::Number(40));
    st.pop();
    st.push(Value::Bool(true)); // [30,true]

    let p=st.pop().unwrap();
    assert_eq!(p.to_string(), "true");

    let pk=st.peek().unwrap(); // [30]
    assert_eq!("30", pk.to_string());

    st.pop();

    let pk=st.peek();
    assert!(pk.is_none());
    assert!(st.is_empty());
}