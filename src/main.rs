// source -> scanner -> tokens -> compile -> bytecode -> vm.interpret(bytecode) -> result 
use nova::nova_repl;
use nova::vm::VM;
use nova::utils::err::*;
use nova::utils::file::run_file;
use nova::data::ops::Value;

use std::env::args;
use std::process::ExitCode;

fn run_main()->Result<()> {
    let cmd_args:Vec<String>=args().collect();
    let argc=cmd_args.len();

    if argc == 1 {
        nova_repl(VM::new())
    } else if argc == 2 {
        let file_name=cmd_args.get(1).unwrap();
        println!("Importing:{file_name}");
        let vm=run_file(&file_name)?;
        nova_repl(vm)
    } else {
        err_other!("Usage: nova [path]")
    }
}

fn main()->ExitCode {
    match run_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err.to_string());
            ExitCode::FAILURE
        }
    }
}