use std::collections::HashMap;
use std::vec;

use crate::scanner::{tokens::*, Scanner};
use crate::data::ops::*;
use crate::utils::err::*;
use crate::data;

pub struct Parser<'src> {
    scanner:Scanner<'src>,
    prev_tok:Option<Token<'src>>,
    curr_tok:Option<Token<'src>>
}

// Parser's job: go from Token stream to a Chunk with all Insts and Consts (compile)

type ParseFn<'src> = fn(&mut Parser<'src>, &mut Chunk);

pub enum Precedence {
    PrecNone,
    PrecOr
}

pub struct ParseRule {
    infix:ParseFn<'static>,
    prec:Precedence
}

impl<'src> Parser<'src> {
    pub fn new<'s>(source:&'s str)->Parser<'s> {
        let scanner=Scanner::new(source);
        Parser { scanner, prev_tok: None, curr_tok: None }
    }

    fn test(&self) {
       let mut v:HashMap<TokenType,ParseRule>=HashMap::new();
    
    //    v.insert(ParseRule {  }, Parser::number);
    //    v.insert(ParseRule {  }, Parser::unary);

    }

    fn number(&mut self, chunk: &mut Chunk)->(){

    }

    fn unary(&mut self, chunk:&mut Chunk) {

    }

    // Err, Err - report consecutive errors until non-err or end
    fn advance(&mut self) {
        // parser.prev = parser.current
        if let Some(t) = self.curr_tok.clone() {
            self.prev_tok.replace(t);
        }

        while let Some(tok) = self.scanner.next() {
            self.curr_tok.replace(tok); // current = next token
            if !tok.is_err() {
                break;
            }
            // report error using self.curr_tok
            println!("Err");
        }
    }

    // EOF is implicit so consume means we expect some actual token type
    fn consume(&mut self, ty:TokenType)->Result<()>{
        if let Some(tok) = self.curr_tok {
            if tok.token_type.eq(&ty) {
                self.advance();
                Ok(())
            } else {
                let msg=format!("Expected {} but got {}", ty, tok);
                errc!(msg)
            }
        } else {
            errc!("Expected {} but got end of input.", ty)
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk)->Result<()> {
        // at first: only exprs

        // advance()
        // expression()
        // consume(EOF, expect end of expr)
        Ok(())
    }
}

/*
    Parser::advance() => set previous to current, set current to next scan token
    advance returns Result<()> (InterpretErr for err msgs)
    error helpers: take the parser.current to report error

    Compiling chunk: a ref to the chunk being compiled (can change over time)

    compile(source, *chunk):
        initScanner(source)       
        compilingChunk = chunk

        advance()
        expression()

        consume(EOF, Expect end of expr)
        endCompiler() => chunk.add OPRETURN

        error reporting
*/


/*
We map each token type to a different kind of expression. We define a function for each expression that outputs the appropriate bytecode. 
Then we build an array of function pointers. The indexes in the array correspond to the TokenType enum values,
and the function at each index is the code to compile an expression of that token type.


*/