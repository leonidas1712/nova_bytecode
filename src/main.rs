use nova::data::ops::Value;
// source -> scanner -> tokens -> compile -> bytecode -> vm.interpret(bytecode) -> result 
use nova::nova_repl;
use nova::vm::VM;
use nova::utils::err::*;
use nova::utils::file::run_file;

use std::env::args;
use std::process::ExitCode;
use std::vec;

fn run_main()->Result<()> {
    let mut r=vec![Value::Number(1), Value::Number(2)];
    r[0]=Value::Number(3);
    dbg!(r[0]);

    let cmd_args:Vec<String>=args().collect();
    let argc=cmd_args.len();

    let mut vm=VM::new();
    if argc == 1 {
        nova_repl(vm)
    } else if argc == 2 {
        let file_name=cmd_args.get(1).unwrap();
        println!("Importing:{file_name}\n");

        run_file(&file_name, &mut vm)?;
        nova_repl(vm)
    } else {
        err_other!("Usage: nova [path]")
    }
}

fn main()->ExitCode {
    #[cfg(debug_assertions)]
    nova::init_logger();

    match run_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err.to_string());
            ExitCode::FAILURE
        }
    }
}