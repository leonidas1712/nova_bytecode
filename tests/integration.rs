use nova::{test_input, test_input_many};

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
fn vm_test() {
    // test_input("x=5", "2");
}