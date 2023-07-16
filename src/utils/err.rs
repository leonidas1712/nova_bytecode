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
        
        InterpretErr::Parse($msg.to_string())
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        InterpretErr::Parse(format!($msg,$($arg),*))
    };
}

#[macro_export]
macro_rules! errc {
    ($msg:expr) => {
        
        Err(errc_i!($msg))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(errc_i!($msg,$($arg),*))
    };
}

#[macro_export]
macro_rules! err_other_i {
    ($msg:expr) => {
        
        InterpretErr::Other($msg.to_string())
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        InterpretErr::Other(format!($msg, $($arg),*))
    };
}

#[macro_export]
macro_rules! err_other {
    ($msg:expr) => {
        
        Err(err_other_i!($msg))
    };

    ($msg:expr $(,$arg:expr),*) => {
        
        Err(err_other_i!($msg, $($arg),*))
    };
}


pub use errn;
pub use errn_i;
pub use errc;
pub use errc_i;
pub use err_other;
pub use err_other_i;


use std::fmt::Display;

// #[derive(Debug)]
pub enum InterpretErr {
    Parse(String),
    Runtime(String),
    Other(String)
}

impl Display for InterpretErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg=match self {
            Self::Parse(m) => format!("(ParseError) {}", m),
            Self::Runtime(m) => format!("(RuntimeError) {}", m),
            Self::Other(m) => m.to_string()
        };

        write!(f, "{}", msg)
    }
}

impl std::fmt::Debug for InterpretErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


pub type Result<T> = anyhow::Result<T, InterpretErr>;