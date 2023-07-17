use nova::{get_output, test_input, test_input_many, 
    vm::VM, parser::parser::Parser, data::ops::Chunk,
    utils::file::run_file
};

#[test]
fn test_binary_ops() {
    let v = vec![
        ("1","1"),
        ("1 + 2 - 3 * 5 + 6 / 9", "-12"),
        ("3 * 4 + 7 - 8 / 2", "15"),
        ("10 - 2 * 4 + 6 / 3", "4"),
        ("8 / 2 - 1 + 5 * 2", "13"),
        ("6 + 9 / 3 - 4 * 2", "1"),
        ("2 * 5 - 3 + 8 / 4", "9"),
        ("5 + 3 / 6 - 2 * 4", "-3"),
        ("\"abc\" + \"def\"", "\"abcdef\"")
    ];

    test_input_many(&v);
}

#[test]
fn test_assignment() {
    // test_input("x=5", "2");
    let inp="x=5;";
    let mut vm=VM::new();
    let res=vm.interpret_with_reset(inp, false);
    assert_eq!(res.unwrap().to_string(), "()");

    assert_eq!(vm.get_global_value("x").unwrap().to_string(), "5");


    let res=vm.interpret_with_reset("let x = 10;", false);
    assert_eq!(res.unwrap().to_string(), "()"); // no return

    assert_eq!(vm.get_global_value("x").unwrap().to_string(), "10");

    let res=vm.interpret_with_reset("x=20; y=30; x+y", false);
    assert_eq!(res.unwrap().to_string(), "50");

    let code="let x = (10+20)*30;  let y=30+50; x+(y*2+3)";
    test_input(code, "1063");

    let examples = vec![
        ("let x = 10; x", "10"),
        ("let a = 3; let b = a * 2; let c = b + a; c / 2", "4"),
        ("let x = 5; let y = x * 2; let z = y - x; z + 10", "15"),
        ("let p = 7; let q = p * 3; let r = q - p; r * 2", "28"),
        ("let a = 2; let b = 5; let c = a + b; let d = c * 3; d / 2", "10"),
    ];
    test_input_many(&examples);
}


#[test]
fn test_concat() {
    let mut c2=Chunk::new();

    // Value::ValObj(Object::new("hi"))
    // Value::ValObj(Object::new(Function{...}))

    test_input("\"hi\" + \"hello\"", "\"hihello\"");
    test_input("\"\" + \"\"", "\"\"");
    test_input("\"hi\" + \"\"", "\"hi\"");
    test_input("\"\" + \"hi\"", "\"hi\"");
}

#[test]
fn test_file() {
    let mut vm=&mut VM::new();
    run_file("test.txt", vm).unwrap();
    assert_eq!(vm.get_global_value("m").unwrap().to_string(), "5");
    assert_eq!(vm.get_global_value("x").unwrap().to_string(), "8");
}