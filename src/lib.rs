extern crate rustyline;
#[macro_use]
extern crate lazy_static;

pub mod data;
pub mod utils;
pub mod vm;
pub mod compiler;
pub mod scanner;
pub mod parser;

use utils::file::run_file;
use vm::VM;
use rustyline::{DefaultEditor, error::ReadlineError};

use utils::constants::*;
use utils::err::*;

use log::{LevelFilter};


use std::io::Write;
fn prt(stdout:&mut dyn Write, s:&str) {
    writeln!(stdout, "{}", s).unwrap();
}

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_module("nova", LevelFilter::Debug)
        .format(|buf,rec| {
            writeln!(buf, "[{} {}] [line {}] {}",
            rec.level(),
            rec.file().unwrap_or("unknown file"),
            rec.line().unwrap_or(0),
            rec.args()
            )
        })
        .init();
}

pub fn process_cmd(cmd:&str, vm:&mut VM) {
    let cmd=cmd.to_string();
    let mut cmd=cmd.split(" ");

    let cmd_name=cmd.next();

    if cmd_name.is_none() {
        println!("Empty command");
    }

    let cmd_name=cmd_name.unwrap();

    match cmd_name {
        "vm" => {
            println!("Process vm");
            println!("{:?}", vm);
        },
        "import" | "run"=> {
            let arg=cmd.next();
            if arg.is_none() {
                println!("No file specified.");
                return;
            }
            let arg=arg.unwrap();

            let res=run_file(&arg, vm);
            if res.is_err() {
                let res=res.unwrap_err().to_string();
                println!("Error when importing file '{}': {}", arg, res);
            } else {
                let res=res.unwrap();
                println!("{}", vm.print_value(res));
            }
        },
        _ => {
            println!("Unknown command: {}", cmd_name)
        }
    }
}


pub fn nova_repl(mut vm:VM)->Result<()> {
    let mut rl = DefaultEditor::new().unwrap();

    println!();
    println!("Welcome to Nova: a highly expressive, dynamically typed functional programming language.\nType an expression to get started.\n");

    loop {
        let readline = rl.readline(">>> ");

        match readline {
            Ok(inp) => {
                let inp = inp.trim().to_string();
                if inp.len() == 0 {
                    continue;
                }

                if QUIT_STRINGS.contains(&inp.as_str()) {
                    println!("See you again!");
                    break;
                }

                if ["cl", "clear"].contains(&inp.as_str()) {
                    let _ = rl.clear_screen();
                    continue;
                }

                rl.add_history_entry(inp.clone().trim()).unwrap();

                if inp.starts_with(CMD_PREFIX) {
                    let cmd= &inp[CMD_PREFIX.len()..];
                    process_cmd(cmd, &mut vm);
                    continue;
                }


                match vm.interpret_with_reset(&inp, false) {
                    Ok(val) => {
                        if !val.is_unit() {
                            println!("{}", vm.print_value(val));
                        }
                    },

                    Err(err) => {
                        println!("{}", err.to_string());
                    }
                }               
            }

            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("See you again!");
                break;
            }
            _ => (),
        }
    }
    Ok(())
}  


// Test helperss
use crate::data::ops::Chunk;
use crate::parser::parser::Parser;

/// Output from running inp
pub fn get_output(inp:&str)->String {
    let mut vm=VM::new();
    let res=vm.interpret(inp);

    match res {
        Ok(val) => vm.print_value(val),
        Err(err) => err.to_string()
    }
}

pub fn test_input(inp:&str, exp:&str) {
    assert_eq!(get_output(inp), exp);
}

pub fn test_input_many(v:&Vec<(&str, &str)>) {
    for (lhs,rhs) in v.iter() {
        test_input(&lhs, &rhs);
    }
}

#[cfg(test)]
pub mod tests {
    use crate::data::ops::*;
    use crate::utils::misc::calc_hash;
    use crate::vm::VM;
    use log::*;

    #[test]
    fn test_stack_ops() {
        let mut c2=Chunk::new();
    
        let idx=c2.add_constant(Value::Number(2), 1);
        let idx2=c2.add_constant(Value::Number(3), 1);
        let idx3=c2.add_constant(Value::Number(5), 1);
    
        c2.write_op(Inst::OpConstant(idx), 1);
        c2.write_op(Inst::OpConstant(idx2), 1);
        c2.write_op(Inst::OpAdd, 1); // 2+3
    
        c2.write_op(Inst::OpConstant(idx3), 1);
        c2.write_op(Inst::OpDiv, 1); // 5/5 = 1
    
        c2.write_op(Inst::OpNegate, 1); // -1
    
        c2.write_op(Inst::OpConstant(idx3), 1);
        c2.write_op(Inst::OpMul, 1); // -1*5=-5
        
    
        c2.write_op(Inst::OpReturn, 1);
    
        println!("{}", c2);
        
        let mut vm=VM::new();
        let res=vm.run(&mut c2, true).unwrap();
    
        assert_eq!(res.to_string(), "-5");
        
    }

    #[test]
    pub fn test() {
        let mut chunk=Chunk::new();
        chunk.write_constant(Value::Number(1), 1);
        chunk.write_constant(Value::Number(2), 1);

        chunk.write_op(Inst::OpReturn, 1);

        let mut vm=VM::new();
        let res=vm.run(&mut chunk, true);
    }
}
