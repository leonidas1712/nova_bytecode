use crate::scanner::{tokens::*, Scanner, self};
use crate::data::ops::*;
use crate::utils::err::*;

pub struct Parser<'src> {
    scanner:Scanner<'src>,
    prev_tok:Option<Token<'src>>,
    curr_tok:Option<Token<'src>>
}

// Parser's job: go from Token stream to a Chunk with all Insts and Consts (compile)
impl<'src> Parser<'src> {
    pub fn new<'s>(source:&'s str)->Parser<'s> {
        let scanner=Scanner::new(source);
        Parser { scanner, prev_tok: None, curr_tok: None }
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
        consume(EOF, Expect end of expr)
        endCompiler() => chunk.add OPRETURN

        error reporting
*/