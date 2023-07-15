use std::fmt::Display;
use std::str::Chars;
use std::iter::Peekable;

use crate::utils::constants::*;

#[derive(Debug)]
pub enum TokenType {
    // Single char
    TokenLeftParen,
    TokenRightParen,
    TokenLeftBrace,
    TokenRightBrace,
    TokenComma,
    TokenDot,
    TokenMinus,
    TokenPlus,
    TokenSemiColon,
    TokenSlash,
    TokenStar,

    // Keywords
    TokenPrint,
    TokenReturn,
    TokenVar,
    TokenIdent,

    // Literals
    TokenNumber,
    TokenFloat,

    // err
    TokenError,
}

use TokenType::*;

// start:0, curr:1
// prt
    // start:0, 

#[derive(Debug)]
pub struct Token<'src> {
    token_type:TokenType,
    pub content:&'src str,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}('{}')", self.token_type, self.content)
    }
}

pub struct Scanner<'src> {
    source:&'src str,
    chars:Peekable<Chars<'src>>,
    start:usize, // index in source for start of curr lexeme
    current:usize, // index of current char
    line:usize // line_num
}

impl<'src> Scanner<'src> {
    pub fn new<'source>(source:&'source str)->Scanner<'source>{
        let chars=source.chars().peekable();
        Scanner { source, chars, start: 0, current: 0, line: 1 }
    }
    // increment, return next char (as &str)
        // str slice of next k chars (k=1)
    fn advance(&mut self)->Option<char>{
        self.current+=1;
        self.chars.next()
    }

    // advance iterator while pred(char) true
    fn advance_while<F>(&mut self, pred:F) where F:Fn(char)->bool {
        while let Some(pk) = self.chars.peek() {
            if pred(*pk) {
                self.advance();
            } else {
                break;
            }
        }   
    }

    // token lifetime tied to source string not self
    fn make_token(&mut self, token_type:TokenType)->Token<'src> {
        let content=&self.source[self.start..self.current];
        self.start=self.current;
        Token { token_type, content }
    }

    // call when peek is ascii digit
    fn number(&mut self)->Token<'src> {
        self.advance_while(|char| char.is_ascii_digit());

        let mut float=false;

        if let Some(pk) = self.chars.peek() {
            if pk==&DOT {
                self.advance();
                self.advance_while(|char| char.is_ascii_digit());
                float=true;
            }
        }

        self.make_token(if float { TokenFloat } else { TokenNumber })
    }

    fn skip_whitespace(&mut self) {
        self.advance_while(|char| char.is_ascii_whitespace());
    }

    // collect consumed into String repr for debugging
    fn serialize(self)->String {
        let s=self.into_iter().map(|tok| tok.to_string()).collect::<Vec<String>>().join(",");
        format!("[{s}]")
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        self.start=self.current;

        if self.start >= self.source.len() {
            return None;
        }

        let nxt=self.advance();
        
        let mut make=|tok_type| Some(self.make_token(tok_type));

        match nxt {
            Some(char) => {
                match char {
                    OPEN_EXPR => make(TokenLeftParen),
                    CLOSE_EXPR => make(TokenRightParen),
                    STMT_END => make(TokenSemiColon),
                    COMMA => make(TokenComma),
                    DOT => make(TokenDot),
                    PLUS => make(TokenPlus),
                    MINUS => make(TokenMinus),
                    SLASH => make(TokenSlash),
                    STAR => make(TokenStar),
                    char if char.is_ascii_digit() => Some(self.number()),
                    _ => make(TokenIdent)
                }
            },

            None => make(TokenError) // err since OOB for start already checked
        }
    }
}

#[test]
fn test_scanner() {
    let inp="(2345) 23";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenLeftParen('('),TokenNumber('2345'),TokenRightParen(')'),TokenNumber('23')]");

    
    let inp="  30   40 \n 50   \t 60 \r   700.30  ";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('30'),TokenNumber('40'),TokenNumber('50'),TokenNumber('60'),TokenFloat('700.30')]");
}


// scanner.start: pointer to start of current lexeme being scanned
// scanner.current = current char being looked at
// scanner.line: current line

// parser: 
    // parser.current:Token => current token
    // prev: prev Token

// ParseRule: either infix or prefix (each with associated fn), and precedence
// array of parse rules: initialise once
// ParseRule ParseFn
    // ParseFn: uses parser.previous, parser.current, ParseRules
    // also needs to be able to write_op
    // needs &mut chunk, &mut parser