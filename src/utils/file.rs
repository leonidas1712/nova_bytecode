extern crate shellexpand;

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::utils::err::*;
use crate::vm::VM;

// full name with ~ expanded
fn get_full_path(filename: &str) -> PathBuf {
    let file_path = shellexpand::tilde(filename).to_string();
    let file_path = Path::new(&file_path).to_owned();
    file_path
}

fn read_file(filename: &str) -> Result<String> {
    let file_path = get_full_path(filename);
    let read = read_to_string(file_path);

    match read {
        Ok(file_string) => Ok(file_string),
        Err(_) => errc!("File '{}' doesn't exist.", filename),
    }
}

use crate::data::ops::Value;
pub fn run_file(filename:&str, vm:&mut VM)->Result<Value> {
    let source=read_file(filename)?;
    // dont reset vm
   vm.interpret_with_reset(&source, false)
}