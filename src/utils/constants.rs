extern crate lazy_static;

use std::collections::HashMap;

use crate::scanner::tokens::*;
use crate::parser::*;

use super::trie::Trie;

// Single char tokens
pub const OPEN_EXPR: char = '(';
pub const CLOSE_EXPR: char = ')';
pub const NEWLINE: char = '\n';
pub const TAB: char = '\t';
pub const VAR_SEP: char = ',';
pub const OPEN_LIST: char = '[';
pub const CLOSE_LIST: char = ']';
pub const SPACE: char = ' ';
pub const EMPTY: char = '\0';
pub const DOT:char='.';
pub const STMT_END:char = ';';
pub const COMMA:char=',';

pub const PLUS:char='+';
pub const SLASH:char='/';
pub const STAR:char='*';
pub const MINUS:char='-';

pub const EQ:char='=';
pub const BANG:char='!';
pub const LESS_THAN:char='<';
pub const GT_THAN:char='>';

pub const OPEN_STRING:char='"';

pub const LEFT_BRACE:char='{';
pub const RIGHT_BRACE:char='}';

pub const UNDERSCORE:char='_';
pub const INFIX:char='$';

// multi char tokens
pub const EQ_EQ:&str="==";
pub const NOT_EQ:&str="!=";
pub const LT_EQ:&str="<=";
pub const GT_EQ:&str=">=";

// keywords
// Keywords
// TokenPrint,
// TokenReturn,
// TokenIf,
// TokenElse,
// TokenTrue,
// TokenFalse,
// TokenAnd,
// TokenOr,
// TokenPipe,
// TokenFunc,
// TokenLet,
pub const TOKEN_PRINT: &str = "print";
pub const TOKEN_RETURN: &str = "return";
pub const TOKEN_IF: &str = "if";
pub const TOKEN_ELSE: &str = "else";
pub const TOKEN_TRUE: &str = "true";
pub const TOKEN_FALSE: &str = "false";
pub const TOKEN_AND: &str = "and";
pub const TOKEN_OR: &str = "or";
pub const TOKEN_PIPE: &str = ">>";
pub const TOKEN_LAMBDA:&str="->";
pub const TOKEN_FUNC: &str = "fun";
pub const TOKEN_LET: &str = "let";


// keywords trie
fn setup_keywords()->Trie{
    let mut trie=Trie::new();

    trie.add_key(OPEN_EXPR, TokenLeftParen);
    trie.add_key(CLOSE_EXPR, TokenRightParen);
    trie.add_key(LEFT_BRACE, TokenLeftBrace);
    trie.add_key(RIGHT_BRACE, TokenRightBrace);

    trie.add_key(STMT_END, TokenSemiColon);
    trie.add_key(COMMA, TokenComma);
    trie.add_key(DOT, TokenDot);
    trie.add_key(PLUS, TokenPlus);
    trie.add_key(MINUS, TokenMinus);
    trie.add_key(SLASH, TokenSlash);
    trie.add_key(STAR, TokenStar);

    // comp
    trie.add_key(EQ, TokenEqual);
    trie.add_key(LESS_THAN, TokenLess);
    trie.add_key(GT_THAN, TokenGt);
    trie.add_key(BANG, TokenNot);


    // two char
    trie.add_key(EQ_EQ, TokenEqEq);
    trie.add_key(NOT_EQ, TokenNotEq);
    trie.add_key(LT_EQ, TokenLessEq);
    trie.add_key(GT_EQ, TokenGtEq);

    // keywords
    trie.add_key(TOKEN_PRINT, TokenPrint);
    trie.add_key(TOKEN_RETURN, TokenReturn);
    trie.add_key(TOKEN_IF, TokenIf);
    trie.add_key(TOKEN_ELSE, TokenElse);
    trie.add_key(TOKEN_TRUE, TokenTrue);
    trie.add_key(TOKEN_FALSE, TokenFalse);
    trie.add_key(TOKEN_AND, TokenAnd);
    trie.add_key(TOKEN_OR, TokenOr);
    trie.add_key(TOKEN_PIPE, TokenPipe);
    trie.add_key(TOKEN_LAMBDA, TokenLambda);
    trie.add_key(TOKEN_FUNC, TokenFunc);
    trie.add_key(TOKEN_LET, TokenLet);
    trie.add_key(INFIX, TokenInfix);


    trie
}

lazy_static! {
    pub static ref KEYWORDS_TRIE:Trie = {
        let trie=setup_keywords();
        trie
    };

    // type of token => rule to parse the token
    pub static ref PARSE_RULE_TABLE:HashMap<TokenType,ParseRule> = {
        let mut m:HashMap<TokenType,ParseRule>=HashMap::new();
        let rules=vec![
            (TokenInteger, ParseRule::new(Some(ParseNumber), None, PrecNone)),
            (TokenMinus, ParseRule::new(Some(ParseUnary), None, PrecNone)),
        ];

        for rule in rules {
            let (ty,rule)=rule;
            m.insert(ty, rule);
        }
    
        m
    };
}


// Keywords
pub const LET_NAME: &str = "let";
pub const IF_NAME: &str = "if";
pub const FN_NAME: &str = "def";

// Operations
pub const ADD: &str = "add";
pub const MULT: &str = "mul";
pub const SUB: &str = "sub";
pub const DBL: &str = "dbl";
pub const INC: &str = "succ";
pub const DEC: &str = "pred";
pub const EQUALS: &str = "eq";
pub const PUTS: &str = "puts";
pub const PRINT: &str = "prt";
pub const OR: &str = "or";
pub const AND: &str = "and";
pub const IMPORT: &str = "import";
pub const CHAIN: &str = ">";
pub const SET: &str = "set";
pub const GET: &str = "get";
pub const LT: &str = "lt";
pub const GT: &str = "gt";
pub const MOD: &str = "mod";
pub const DIV: &str = "div";

// Boolean
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";

// List
pub const CONS: &str = "cons";
pub const CAR: &str = "car";
pub const CDR: &str = "cdr";
pub const LCONS: &str = "lcons";
pub const LCDR: &str = "lcdr";
pub const LCAR: &str = "lcar";
pub const INDEX: &str = "idx";
pub const EMPTY_LIST: &str = "[]";

// builtins list
pub const BUILTINS: [&'static str; 26] = [
    ADD, MULT, SUB, DBL, INC, DEC, EQUALS, PUTS, PRINT, OR, AND, IMPORT, CHAIN, SET, GET, LT, GT,
    MOD, DIV, CONS, CAR, CDR, LCONS, LCDR, LCAR, INDEX,
];

// Lambda
// pub const LAMBDA: &str = "->";
pub const LAMBDA_TYPE: &str = "lambda";

// Binary operations
pub const COMP_OPR: &str = "$";
pub const COMP_LEFT: &str = "@";
pub const PIPE: &str = ">>";

// Some useful token arrays
// pub const SPLIT_TOKENS: [&'static str; 12] = [
//     OPEN_EXPR, CLOSE_EXPR, NEWLINE, TAB, VAR_SEP, OPEN_LIST, CLOSE_LIST, SPACE, LAMBDA, COMP_OPR,
//     PIPE, STMT_END,
// ];

// pub const DONT_ADD: [&'static str; 5] = [NEWLINE, TAB, VAR_SEP, SPACE, EMPTY];

// pub const OPEN_TOKENS: [&'static str; 2] = [OPEN_EXPR, OPEN_LIST];
// pub const CLOSE_TOKENS: [&'static str; 2] = [CLOSE_EXPR, CLOSE_LIST];

// pub const EXPR_TUP: (&'static str, &'static str) = (OPEN_EXPR, CLOSE_EXPR);
// pub const LIST_TUP: (&'static str, &'static str) = (OPEN_LIST, CLOSE_LIST);

// ASTNode types
pub const EXPRESSION: &str = "expression";
pub const LIST: &str = "list";
pub const SYMBOL: &str = "symbol";
pub const NUMBER: &str = "number";
pub const STRING: &str = "string";

// REPL commands
pub const QUIT_STRINGS: [&'static str; 4] = ["quit", "quit()", "exit", "exit()"];
pub const FAT_ARROW: &str = "=>";
pub const CMD_PREFIX: &str = ":";

pub const COMMENT: &str = "#";

// pub const RESERVED_KEYWORDS: [&'static str; 36] = [
//     LET_NAME,
//     FN_NAME,
//     IF_NAME,
//     EQUALS,
//     PUTS,
//     PRINT,
//     OR,
//     AND,
//     IMPORT,
//     CHAIN,
//     CONS,
//     SET,
//     GET,
//     CAR,
//     CDR,
//     LAMBDA,
//     VAR_SEP,
//     OPEN_EXPR,
//     CLOSE_EXPR,
//     OPEN_LIST,
//     CLOSE_LIST,
//     TRUE,
//     FALSE,
//     AND,
//     OR,
//     LCAR,
//     LCDR,
//     LCONS,
//     GT,
//     LT,
//     LAMBDA_TYPE,
//     COMP_OPR,
//     COMP_LEFT,
//     PIPE,
//     FAT_ARROW,
//     CMD_PREFIX,
// ];