extern crate anyhow;

#[macro_export]
macro_rules! errn_i {
    ($msg:expr) => {
        
        InterpretErr::Runtime($msg.to_string())
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        InterpretErr::Runtime(format!($msg, $($arg),*))
    };
}

#[macro_export]
macro_rules! errn {
    ($msg:expr) => {
        
        Err(errn_i!($msg))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(errn_i!($msg, $($arg),*))
    };
}

#[macro_export]
macro_rules! errc_i {
    ($msg:expr) => {
        
        InterpretErr::Compile($msg.to_string())
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        InterpretErr::Compile(format!($msg, $($arg),*))
    };
}

#[macro_export]
macro_rules! errc {
    ($msg:expr) => {
        
        Err(errc_i!($msg))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(errc_i!($msg, $($arg),*))
    };
}

use anyhow::Error;
pub (crate) use errn;
pub (crate) use errc;
pub (crate) use errn_i;
pub (crate) use errc_i;


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


pub (crate) type Result<T> = anyhow::Result<T, InterpretErr>;