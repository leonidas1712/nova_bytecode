use nova::data::ops::Value;
use nova::data::stack::STACK_SIZE;
// source -> scanner -> tokens -> compile -> bytecode -> vm.interpret(bytecode) -> result 
use nova::nova_repl;
use nova::vm::VM;
use nova::utils::err::*;
use nova::utils::file::run_file;

use std::env::args;
use std::process::ExitCode;


fn run_main()->Result<()> {
    let mut stack:[Option<u32>; 10]=[None;10];
    // stack[x]=Some(100);
    // dbg!(stack[4]);




    let cmd_args:Vec<String>=args().collect();
    let argc=cmd_args.len();

    let mut vm=VM::new();
    if argc == 1 {
        nova_repl(vm)
    } else if argc >= 2 {
        let file_name=cmd_args.get(1).unwrap();
        println!("Importing:{file_name}\n");

        let no_shell=cmd_args.get(2);

        run_file(&file_name, &mut vm)?;

        if let Some(tok) = no_shell {
            if !tok.eq("-o") {
                nova_repl(vm)?;
            }
        }

        Ok(())
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