use std::collections::HashMap;
use std::fmt::format;
use std::vec;

use crate::scanner::{tokens::*, Scanner};
use crate::data::ops::*;
use crate::utils::constants::PARSE_RULE_TABLE;
use crate::utils::err::*;
use crate::data::*;


#[derive(Clone, Copy)]
pub enum Precedence {
    PrecNone,
    PrecAssign,
}

pub use Precedence::*;

// TokenType -> ParseRule
#[derive(Clone, Copy)]
pub enum RuleType {
    RuleInfix,
    RulePrefix
}

pub use RuleType::*;

// type ParseFn<'src> = fn(&mut Parser<'src>, &mut Chunk)->Result<()>;

#[derive(Clone, Copy)]
pub enum ParseFn {
    ParseNumber
}

pub use ParseFn::*;

#[derive(Clone, Copy)]
pub struct ParseRule {
    rule_type:RuleType,
    func:ParseFn,
    prec:Precedence
}

impl ParseRule {
    pub fn new(rule_type:RuleType, func:ParseFn, prec:Precedence)->ParseRule {
        ParseRule {
            rule_type,
            func,
            prec
        }
    }

    // change to tup for associativity etc
    pub fn get_precedence(&self)->usize {
        match self.prec {
            PrecNone => 1,
            PrecAssign => 2
        }
    }

    fn get_rule(ty:TokenType)->Option<ParseRule>{
        PARSE_RULE_TABLE.get(&ty).copied()
    }
}

#[derive(Debug)]
pub struct Parser<'src> {
    scanner:Scanner<'src>,
    prev_tok:Option<Token<'src>>,
    curr_tok:Option<Token<'src>>,
    line:usize
}

// Parser's job: go from Token stream to a Chunk with all Insts and Consts (compile)
impl<'src> Parser<'src> {
    pub fn new<'s>(source:&'s str)->Parser<'s> {
        let scanner=Scanner::new(source);
        Parser { scanner, prev_tok: None, curr_tok: None, line:1 }
    }

    fn get_prev(&self)->Result<Token<'src>> {
        match self.prev_tok {
            Some(tok) => {
                Ok(tok)
            },
            // report always returns Err
            None =>  Err(self.report_msg(Token::err(self.line), "Expected a token").unwrap_err())
        }
    }

    // ParseFn: assume that the token to parse is set in self.prev
    // add err handling

    // get_prev()?; 

    // expect_token_type(ty)->Result<()>
    pub fn number(&mut self, chunk: &mut Chunk)->Result<()>{
        let prev=self.get_prev()?;
        self.expect_token_type(prev, TokenInteger, "integer")?; // only errs when bug in parser

        // convert to number
        let value:IntType = prev.content.parse().unwrap();
        let value=Value::Number(value);

        let idx=chunk.add_constant(value, prev.line);
        chunk.write_op(Inst::OpConstant(idx), prev.line);

        Ok(())
    }

    pub fn unary(&mut self, chunk:&mut Chunk)->Result<()>{
        Ok(())
    }

    fn expression(&mut self, chunk:&mut Chunk)->Result<()>{
        self.parse_precedence(chunk, PrecAssign)?;
        Ok(())
    }

    // call based on enum
    fn call_parse_fn(&mut self, chunk:&mut Chunk, ty:ParseFn)->Result<()>{
        match ty {
            ParseNumber => self.number(chunk),
        }
    }

    // why do we expect expression for prefix rule

    fn parse_precedence(&mut self, chunk: &mut Chunk, prec:Precedence)->Result<()> {
        self.advance()?;
        // get rule based on parser.prev.type
        let prev=self.get_prev()?;

        // no rule exists , then prefix or not
        let rule=ParseRule::get_rule(prev.token_type);

        if rule.is_none() {
            let msg=format!("Unrecognised token: {}", prev.to_string());
            return self.report_msg(prev, msg);
        }

        let rule=rule.unwrap();

        match rule.rule_type {
            RulePrefix => {
                // use rule to call function
                self.call_parse_fn(chunk, rule.func)?;
                Ok(())
            },
            _ => self.report_err("Expect expression")
        }
    }

    // Err, Err - report consecutive errors until non-err or end
    // advance curr/prev ptrs - should do Result for err
    pub fn advance(&mut self)->Result<()> {
        // is_done true after this
        if self.at_last() {
            self.curr_tok.take();
            return Ok(());
        }

        // parser.prev = parser.current
        if let Some(t) = self.curr_tok.clone() {
            self.prev_tok.replace(t);
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

        self.advance();
        self.expression(chunk)?;
        self.end_compile(chunk);
        // consume(EOF, expect end of expr)
        Ok(())
    }

    pub fn end_compile(&mut self, chunk:&mut Chunk) {
        chunk.write_op(Inst::OpReturn, self.line);
    }

    // when p=c=last token
    fn at_last(&self)->bool {
        let k=self.curr_tok.zip(self.prev_tok);
        k.map(|t| t.0==t.1).unwrap_or(false)
    }

    pub fn is_done(&self)->bool {
        self.curr_tok.is_none()
    }

    // Error Handling

    // always returns err variant
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

        let msg=format!("{} {}: {}", reported_msg, token_part, msg.to_string());
        errc!(msg)
    }

    // without token
    fn report_err<K>(&self, msg:K)->Result<()> where K:ToString {
        self.report_msg(Token::err(self.line), msg)
    }

    // expect_token_type(prev, "number")?;
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
    // let mut p=Parser::new("2 3");
    // p.advance(); // p=None, c=2
    // p.advance(); // p=2, c=3
    // p.advance(); // p=3,c =3
    // p.advance(); // p=3,c =None
    // p.advance(); // p=3,c =None

    // dbg!(&p);
    // dbg!(p.is_done());
 

    // // just put 2
    // // let mut p=Parser::new("2");

}

#[test]
fn test_parse2() {
    let mut p=Parser::new("2");
    let mut chunk=Chunk::new();

    let res=p.compile(&mut chunk);

    dbg!(chunk);
}


/*
We map each token type to a different kind of expression. We define a function for each expression that outputs the appropriate bytecode. 
Then we build an array of function pointers. The indexes in the array correspond to the TokenType enum values,
and the function at each index is the code to compile an expression of that token type.


*/