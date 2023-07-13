#[macro_export]
macro_rules! errn {
    ($msg:expr) => {
        
        Err(InterpretErr::Runtime($msg.to_string()))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(InterpretErr::Runtime(format!($msg, $($arg),*)))
    };
}

#[macro_export]
macro_rules! errc {
    ($msg:expr) => {
        
        Err(InterpretErr::Compile($msg.to_string()))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(InterpretErr::Compile(format!($msg, $($arg),*)))
    };
}

pub (crate) use errn;
pub (crate) use errc;

use std::fmt::Display;

#[derive(Debug)]
pub enum InterpretErr {
    Compile(String),
    Runtime(String)
}

impl Display for InterpretErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg=match self {
            Self::Compile(m) => format!("CompileError -  {}", m),
            Self::Runtime(m) => format!("RuntimeError - {}", m)
        };

        write!(f, "{}", msg)
    }
}

pub (crate) type Result<T> = std::result::Result<T,InterpretErr>;