extern crate lazy_static;


use crate::scanner::tokens::*;

use super::trie::Trie;

// Single char tokens
pub const OPEN_EXPR: char = '(';
pub const CLOSE_EXPR: char = ')';
pub const STRING_QUOTE:char='"';
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

// Keywords
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

// we know for sure the char is static -> this is ok
fn cstr(char:char)->&'static str {
    Box::leak(char.to_string().into_boxed_str())
}

// keywords trie
fn setup_keywords()->Trie<&'static str,TokenType>{
    let mut trie=Trie::new();

    trie.add_key(cstr(OPEN_EXPR), TokenLeftParen);
    trie.add_key(cstr(CLOSE_EXPR), TokenRightParen);
    trie.add_key(cstr(LEFT_BRACE), TokenLeftBrace);
    trie.add_key(cstr(RIGHT_BRACE), TokenRightBrace);
    trie.add_key(cstr(STRING_QUOTE), TokenStringQuote);

    trie.add_key(cstr(STMT_END), TokenSemiColon);
    trie.add_key(cstr(COMMA), TokenComma);
    trie.add_key(cstr(DOT), TokenDot);
    trie.add_key(cstr(PLUS), TokenPlus);
    trie.add_key(cstr(MINUS), TokenMinus);
    trie.add_key(cstr(SLASH), TokenSlash);
    trie.add_key(cstr(STAR), TokenStar);

    // comp
    trie.add_key(cstr(EQ), TokenEqual);
    trie.add_key(cstr(LESS_THAN), TokenLess);
    trie.add_key(cstr(GT_THAN), TokenGt);
    trie.add_key(cstr(BANG), TokenNot);
    trie.add_key(cstr(INFIX), TokenInfix);



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

    trie
}

use std::io;

lazy_static! {
    pub static ref KEYWORDS_TRIE:Trie<&'static str,TokenType> = {
        let trie=setup_keywords();
        trie
    };
}

pub const QUIT_STRINGS: [&'static str; 4] = ["quit", "quit()", "exit", "exit()"];
pub const FAT_ARROW: &str = "=>";
pub const CMD_PREFIX: &str = ":";