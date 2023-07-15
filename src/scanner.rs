use std::fmt::Display;
use std::str::Chars;
use std::iter::Peekable;

use crate::utils::constants::*;

#[derive(Debug,Clone,Copy, PartialEq)]
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
    TokenIf,
    TokenTrue,
    TokenFalse,
    TokenAnd,
    TokenOr,
    TokenIdent,
    TokenPipe,

    // Literals
    TokenNumber,
    TokenFloat,
    TokenString,

    // Comp
    TokenEqual, // =
    TokenEqEq, // ==
    TokenNotEq, // !=
    TokenNot, // !
    TokenLess, // <
    TokenLessEq, // <=
    TokenGt, // >
    TokenGtEq, // >=

    // misc
    TokenComment,
    TokenError,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
    chars:LookaheadChars<'src>,
    start:usize, // index in source for start of curr lexeme
    current:usize, // index of current char
    line:usize, // line_num,
}

// store lookahead of one char i.e the Option<char> after peek
pub struct LookaheadChars<'src> {
    chars:Peekable<Chars<'src>>,
    peek:Option<char> // current peek (chars always points one step ahead of peek)
}

impl<'src> LookaheadChars<'src> {
    pub fn new<'source>(source:&'source str)->LookaheadChars<'source> {
        let mut chars=source.chars().peekable();
        let peek=chars.next();

        LookaheadChars { chars, peek }
    }

    pub fn peek(&self)->Option<char> {
        self.peek
    }

    pub fn peek_next(&mut self)->Option<char> {
        self.chars.peek().map(|c| c.to_owned())
    }
}

impl<'src> Iterator for LookaheadChars<'src> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let nxt=self.peek;
        self.peek=self.chars.next();
        nxt
    }
}

impl<'src> Scanner<'src> {
    pub fn new<'source>(source:&'source str)->Scanner<'source>{
        let chars=LookaheadChars::new(source);
        Scanner { source, chars, start: 0, current: 0, line: 1 }
    }

    pub fn peek(&mut self)->Option<char> {
        self.chars.peek()
    }

    pub fn peek_next(&mut self)->Option<char> {
        self.chars.peek_next()
    }

    // increment, return next char (as &str)
        // str slice of next k chars (k=1)
    fn advance(&mut self)->Option<char>{
        self.current+=1;
        self.chars.next()
    }

    // advance iterator while pred(char) true
    // when this stops, current points to first char where pred was false
    fn advance_while<F>(&mut self, pred:F) where F:Fn(char)->bool {
        while let Some(pk) = self.peek() {
            if pred(pk) {
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

        if let Some(pk) = self.peek() {
            if pk==DOT {
                self.advance();
                self.advance_while(|char| char.is_ascii_digit());
                float=true;
            }
        }

        self.make_token(if float { TokenFloat } else { TokenNumber })
    }

    // when current char is open_string
    fn string(&mut self)->Token<'src> {
        self.start=self.current; // idx already at first char
        self.advance_while(|ch| ch!=OPEN_STRING);

        let tok=self.make_token(TokenString); // start=idx of first char, curr=idx of terminator

        self.advance(); // move past terminator
        tok
    }

    fn skip_whitespace(&mut self) {
        while let Some(pk) = self.peek() {
            match pk {
                NEWLINE => {
                    self.advance();
                    self.line += 1;
                },
                pk if pk.is_ascii_whitespace() => {
                    self.advance();
                },
                SLASH => {         
                    let is_comm=self.peek_next().map(|ch| ch==SLASH).unwrap_or(false);
                    if !is_comm {
                        return;
                    }
                    
                    // advance until \n
                    self.advance_while(|ch| ch!=NEWLINE);
                    self.advance();
                    self.line+=1;
                },
                _ => break
            }
        }
    }

    // return true and advance if peek is char, else false
    fn match_char(&mut self, char:char)->bool {
        if let Some(ch) = self.peek() {
            ch==char
        } else {
            false
        }
    }

    // check next char: if equal to match_next, use if_match and advance. else, just ret else_match
    fn make_two_char(&mut self, match_next:char, if_match:TokenType, else_match:TokenType)->Option<Token<'src>> {
        if let Some(pk) = self.peek() {
            if pk==match_next {
                self.advance();
                return Some(self.make_token(if_match))
            } 
            return Some(self.make_token(else_match))
        }
        None
    }

    // collect consumed into String repr for debugging
    fn serialize(self)->String {
        let s=self.into_iter().map(|tok| tok.to_string()).collect::<Vec<String>>().join(",");
        format!("[{s}]")
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Token<'src>;
    // advance: self.current+=1;
    // make_token: self.start=self.current

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        self.start=self.current;

        if self.start >= self.source.len() {
            return None;
        }

        let nxt=self.advance();
        
        let mut make=|tok_type| Some(self.make_token(tok_type));

        // make_two_char(match_next:char, if_match:TokenType, else_match:TokenType)
        match nxt {
            Some(char) => {
                match char {
                    OPEN_EXPR => make(TokenLeftParen),
                    OPEN_STRING => Some(self.string()),
                    CLOSE_EXPR => make(TokenRightParen),
                    STMT_END => make(TokenSemiColon),
                    COMMA => make(TokenComma),
                    DOT => make(TokenDot),
                    PLUS => make(TokenPlus),
                    MINUS => make(TokenMinus),
                    SLASH => make(TokenSlash),
                    STAR => make(TokenStar),
                    char if char.is_ascii_digit() => Some(self.number()),

                    // two char tokens - can replace with trie search later
                    EQ => self.make_two_char(EQ, TokenEqEq, TokenEqual),
                    BANG => self.make_two_char(EQ, TokenNotEq, TokenNot),
                    LESS_THAN => self.make_two_char(EQ, TokenLessEq, TokenLess), // trie search needed: '<' -> '<' for pipe, '=' for '<='
                    GT_THAN => self.make_two_char(EQ, TokenGtEq, TokenGt),
                    _ => make(TokenIdent)
                }
            },

            None => make(TokenError) // err since OOB for start already checked
        }
    }
}

#[test]
fn test_lookahead() {
    let inp="23";
    let mut s=LookaheadChars::new(inp);
    assert_eq!(s.peek(), Some('2')); // 2
    assert_eq!(s.peek_next(), Some('3')); // 3
    s.next();
 
    assert_eq!(s.peek(), Some('3')); // 3
    assert_eq!(s.peek_next(), None); // None

    s.next();

    assert_eq!(s.peek(), None); // None
    assert_eq!(s.peek_next(), None); // None


    s.next();
    s.next();

    assert_eq!(s.peek(), None); // None
    assert_eq!(s.peek_next(), None); // None
}

#[test]
fn test_scanner() {
    let inp="(2345) 23";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenLeftParen('('),TokenNumber('2345'),TokenRightParen(')'),TokenNumber('23')]");

    
    let inp="  30   40 \n 50   \t 60 \r   700.30  ";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('30'),TokenNumber('40'),TokenNumber('50'),TokenNumber('60'),TokenFloat('700.30')]");

    let inp="xy\ny\nz\ntext";
    let mut s=Scanner::new(inp);
    while let Some(_) = s.peek() {
        s.next();
    }
    assert_eq!(s.line, 4);
}

#[test]
fn test_scanner_two() {
    let inp="4 == 2 = 3 == 5";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('4'),TokenEqEq('=='),TokenNumber('2'),TokenEqual('='),TokenNumber('3'),TokenEqEq('=='),TokenNumber('5')]");

    let inp="!4 != !0 != 5";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNot('!'),TokenNumber('4'),TokenNotEq('!='),TokenNot('!'),TokenNumber('0'),TokenNotEq('!='),TokenNumber('5')]");

    let inp="4 < 5 <= 6 > 5 >= 10 < 9 >=2>8";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('4'),TokenLess('<'),TokenNumber('5'),TokenLessEq('<='),TokenNumber('6'),TokenGt('>'),TokenNumber('5'),TokenGtEq('>='),TokenNumber('10'),TokenLess('<'),TokenNumber('9'),TokenGtEq('>='),TokenNumber('2'),TokenGt('>'),TokenNumber('8')]");
}

#[test]
fn test_comment() {
    let inp="2/3";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('2'),TokenSlash('/'),TokenNumber('3')]");

    let inp="\n\t 2/3 // c1 \n 40 // c2\n  400 // \t another comment \n 50 ";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNumber('2'),TokenSlash('/'),TokenNumber('3'),TokenNumber('40'),TokenNumber('400'),TokenNumber('50')]");
}

#[test]
fn test_string() {
    let inp="2\" some string lit \"3";
    // TokenString("some string..")
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),  "[TokenNumber('2'),TokenString(' some string lit '),TokenNumber('3')]");
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