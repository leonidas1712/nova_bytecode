use std::collections::HashMap;
use std::fmt::format;
use std::vec;

use crate::scanner::{tokens::*, Scanner};
use crate::data::ops::*;
use crate::utils::constants::PARSE_RULE_TABLE;
use crate::utils::err::*;
use crate::{data::*, vm};
use Inst::*;

use super::rules::*;

#[derive(Debug)]
pub struct Parser<'src> {
    scanner:Scanner<'src>,
    prev_tok:Option<Token<'src>>,
    curr_tok:Option<Token<'src>>,
    line:usize
}

/*
    Adding a new parse rule:
    1. 
*/

// Parser's job: go from Token stream to a Chunk with all Insts and Consts (compile)
impl<'src> Parser<'src> {
    pub fn new<'s>(source:&'s str)->Parser<'s> {
        let scanner=Scanner::new(source);
        Parser { scanner, prev_tok: None, curr_tok: None, line:1 }
    }

    // ParseFn: assume that the token to parse is set in self.prev

    // expect_token_type(ty)->Result<()>
    pub fn number(&mut self, chunk: &mut Chunk)->Result<()>{
        let prev=self.expect_prev()?;
        self.expect_token_type(prev, TokenInteger, "integer")?; // only errs when bug in parser

        // convert to number
        let value:IntType = prev.content.parse().unwrap();
        let value=Value::Number(value);

        chunk.write_constant(value, prev.line);
        Ok(())
    }

    // unary called based on rules table
    pub fn unary(&mut self, chunk:&mut Chunk)->Result<()>{
        let prev=self.expect_prev()?;
        self.expression(chunk)?; // next expression result goes onto stack

        match prev.token_type {
            TokenMinus => chunk.write_op(Inst::OpNegate, prev.line),
            _ => ()
        }
        Ok(())
    }

    // binary called based on rules table
    pub fn binary(&mut self, chunk:&mut Chunk)->Result<()>{
        log::debug!("Called binary, curr_tok:{:?}, prev:{:?}", &self.curr_tok, &self.prev_tok);
        let prev=self.expect_prev()?; // operator
        let rule=self.expect_rule(prev)?;
        
        // put right side onto stack - use next higher precedence for left associativity
        self.parse_precedence(chunk, rule.prec.get_next_prec())?;

        if rule.infix.is_none() {
            let msg=format!("Expected operation but got {}", prev);
            return self.report_msg(prev,msg);
        }

        // match on token type
        let op:Inst = match prev.token_type {
            TokenPlus => OpAdd,
            TokenMinus => OpSub,
            TokenStar => OpMul,
            TokenSlash => OpDiv,
            _ => return self.report_msg(prev, "Unrecognised operation")

        };

        chunk.write_op(op, prev.line);
        Ok(())
    }

    fn expression(&mut self, chunk:&mut Chunk)->Result<()>{
        // assign is the lowest valid precedence: other ops can bind as much as possible
        self.parse_precedence(chunk, PrecAssign)?;
        Ok(())
    }

    // call based on enum
    fn call_parse_fn(&mut self, chunk:&mut Chunk, ty:ParseFn)->Result<()>{
        match ty {
            ParseNumber => self.number(chunk),
            ParseUnary => self.unary(chunk),
            ParseBinary => self.binary(chunk),
        }
    }

    fn parse_precedence(&mut self, chunk: &mut Chunk, prec:Precedence)->Result<()> {
        self.advance()?;
        // get rule based on parser.prev.type
        let prev=self.expect_prev()?;

        // no rule exists , then prefix or not
        let rule=self.expect_rule(prev)?;

        // we should first have a prefix (expect prefix)
        let prefix=rule.prefix;
        if prefix.is_none() {
            let msg=format!("Expect expression but got: '{}'", prev.content);
            return self.report_msg(prev, msg);
        }

        let prefix_fn=prefix.unwrap();
        self.call_parse_fn(chunk, prefix_fn)?;

        // // infix down here - pratt parsing
        loop {
            if self.is_done() {
                break;
            }

            let curr_tok=self.expect_current()?;
            let rule=self.expect_rule(curr_tok)?;
            
            if prec.get_precedence_val() > rule.prec.get_precedence_val() {
                break;
            }

             // so that curr = next token after infix in subseq call to parse preced
             // e.g 1+2 : now curr=+, advance => curr = 2, parse preced calls advance => prev=2, get prefix...
            self.advance()?;

            // get infix fn from prev
            let infix=rule.infix;
            if infix.is_none() {
                let msg=format!("Expected operation but got '{}'", curr_tok.content);
                return self.report_msg(curr_tok, msg);
            }

            let infix=infix.unwrap();
            self.call_parse_fn(chunk, infix)?;
        }


        Ok(())
        // self.call_parse_fn(chunk, prefix_fn) // REMOVE
    }

    // Err, Err - report consecutive errors until non-err or end
    // advance curr/prev ptrs - should do Result for err
    pub fn advance(&mut self)->Result<()> {
        // parser.prev = parser.current
        if let Some(t) = self.curr_tok.clone() {
            self.prev_tok.replace(t);
        } else {
            // curr_tok is None so set prev to None also
            self.prev_tok.take();
        }

        // set curr to none if scanner is finished
        if self.scanner.peek().is_none() {
            // println!("Prev:{:?}", self.prev_tok);
            // println!("Curr:{:?}", self.curr_tok);
            self.curr_tok.take();
        }

        // set current tok to next
        while let Some(tok) = self.scanner.next() {
            self.curr_tok.replace(tok); // current = next token
            self.line=tok.line;
            if !tok.is_err() {
                break;
            }
            // report error using self.curr_tok
            return self.report_msg(tok, "Error")
        }

        Ok(())
    }

    // EOF is implicit so consume means we expect some actual token type
    fn consume(&mut self, ty:TokenType)->Result<()>{
        if let Some(tok) = self.curr_tok {
            if tok.token_type.eq(&ty) {
                self.advance();
                Ok(())
            } else {
                let msg=format!("Expected {} but got {}", ty, tok);
                self.report_msg(tok, &msg)
            }
        } else {
            let msg=format!("Expected {} but got end of input.", ty);
            self.report_err(&msg)
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk)->Result<()> {
        // at first: only exprs

        self.advance()?;
        self.expression(chunk)?;
        self.end_compile(chunk);
        // consume(EOF, expect end of expr)
        Ok(())
    }

    pub fn end_compile(&mut self, chunk:&mut Chunk) {
        chunk.write_op(Inst::OpReturn, self.line);
    }

    pub fn is_done(&self)->bool {
        self.curr_tok.is_none()
    }



    // Error Handling

    /// Report error with a reference token to include in string. Always returns err variant
    fn report_msg<K>(&self, token:Token<'_>, msg:K)->Result<()> where K:ToString{
        let mut reported_msg=format!("[line {}] Error", token.line);

        let token_part=if self.is_done() {
            "at end".to_string()
        } else {
            match token.token_type {
                TokenError => String::from(""),
                _ => format!("at '{}'", token.content)
            }
        };

        let msg=format!("{} {} - {}", reported_msg, token_part, msg.to_string());
        errc!(msg)
    }

    /// Report error without a reference token. Always returns err variant
    fn report_err<K>(&self, msg:K)->Result<()> where K:ToString {
        self.report_msg(Token::err(self.line), msg)
    }


     /// Expect that prev is not None
     fn expect_prev(&self)->Result<Token<'src>> {
        match self.prev_tok {
            Some(tok) => {
                Ok(tok)
            },
            // report always returns Err
            None =>  Err(self.report_msg(Token::err(self.line), "Expected a token").unwrap_err())
        }
    }

    /// Expect that current is not None
    fn expect_current(&self)->Result<Token<'src>> {
        match self.curr_tok {
            Some(tok) => {
                Ok(tok)
            },
            // report always returns Err
            None =>  Err(self.report_msg(Token::err(self.line), "Expected a token").unwrap_err())
        }
    }

    /// Expect that this token maps to a rule. Returns error "Unrecognised token" otherwise
    fn expect_rule(&self, token:Token)->Result<ParseRule> {
        let rule=ParseRule::get_rule(token.token_type);

        if rule.is_none() {
            let msg=format!("Unrecognised token: {}", token.content);
            self.report_msg(token, msg)?; // returns out 
        }

        let rule=rule.unwrap();
        Ok(rule) 
    }

    /// Expect that token type matches given type and returns error with expected type_string otherwise
    fn expect_token_type(&self, token:Token<'src>, ty:TokenType, type_string:&str)->Result<()> {
        match token.token_type {
            // err msgs
            token_type if !token_type.eq(&ty) => {
                let msg=format!("Expected a {} but got '{}'", type_string, token.content);
                self.report_err(&msg)
            },
            _ => {
                Ok(())
            }
        }
    }
    // End helpers
}

/**
 
  At the beginning of parsePrecedence(), we look up a prefix parser for the current token.
   The first token is always going to belong to some kind of prefix expression, by definition. 
   It may turn out to be nested as an operand inside one or more infix expressions, but as you 
   read the code from left to right, the first token you hit always belongs to a prefix expression.
 */


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

    expression():
        parsePrecedence(PREC_ASSIGN)

    parsePrecedence():
        advance();
        get prefix rule from table according to parser.previous.tokentype
        run that

    ParseRule: ParseFn prefix, ParseFn infix, Precedence
*/


/*
    Grammar:
    
    expression -> literal | func_defn | assignment | stmt
    assignment
*/

#[test]
fn test_parse() {
    let code="123\n1.0\n2.0";
    let mut p=Parser::new(code);
    let mut chunk=Chunk::new();

    let res=p.compile(&mut chunk);
    dbg!(chunk);

    let mut chunk=Chunk::new();
}

#[test]
fn test_parse2() {
    let mut p=Parser::new("-");
    let mut chunk=Chunk::new();


    let res=p.compile(&mut chunk);
    assert!(res.is_err());
}




/*
We map each token type to a different kind of expression. We define a function for each expression that outputs the appropriate bytecode. 
Then we build an array of function pointers. The indexes in the array correspond to the TokenType enum values,
and the function at each index is the code to compile an expression of that token type.


*/