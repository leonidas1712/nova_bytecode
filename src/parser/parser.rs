use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

use crate::scanner::delim::{Delimiter, DelimiterScanner};
use crate::scanner::{tokens::*, Scanner};
use crate::data::ops::*;
use crate::utils::err::*;

use Inst::*;

use super::rules::*;
use super::rules::ParseRule;

// add stacks to track () and string ""
// open token, close token for each pair
// creates a new stack for each pair and tracks => hashmap

#[derive(Debug)]
pub struct Parser<'src> {
    scanner:Scanner<'src>,
    prev_tok:Option<Token<'src>>,
    curr_tok:Option<Token<'src>>,
    line:usize,
    delim_scanner:DelimiterScanner,
    is_stmt:bool // set to true when semicolon consumed
}

/*
    Adding a new parse rule:
    1. 
*/

// Parser's job: go from Token stream to a Chunk with all Insts and Consts (compile)
impl<'src> Parser<'src> {
    pub fn new<'s>(source:&'s str)->Parser<'s> {
        let scanner=Scanner::new(source);
        let delimiters:Vec<Delimiter> = vec![
            Delimiter::new(TokenLeftParen, TokenRightParen, false),
            Delimiter::new(TokenStringQuote, TokenStringQuote, true)
        ];

        let delim_scanner=DelimiterScanner::new(delimiters);

        Parser { scanner, prev_tok: None, curr_tok: None, line:1, delim_scanner, is_stmt:false }
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
        // next expression result goes onto stack
        // PrecUnary higher than binary => -1+2 means - will bind 1 and prevent + from consuming
        self.parse_precedence(chunk, PrecUnary)?; 

        match prev.token_type {
            TokenMinus => chunk.write_op(Inst::OpNegate, prev.line),
            _ => ()
        }
        Ok(())
    }

    // binary called based on rules table
    pub fn binary(&mut self, chunk:&mut Chunk)->Result<()>{
        // log::debug!("Called binary, curr_tok:{:?}, prev:{:?}", &self.curr_tok, &self.prev_tok);
        let prev=self.expect_prev()?; // operator
        // let rule=self.expect_rule(prev)?;
        let rule=ParseRule::get_rule(prev.token_type);
        
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

    fn grouping(&mut self, chunk:&mut Chunk)->Result<()> {
        self.expression(chunk)?;
        self.consume(TokenRightParen)?;
        Ok(())
    }

    // curr should be TokenString
    // advance so that curr is right past ending quote
    fn string(&mut self, chunk: &mut Chunk)->Result<()> {
        let string=self.consume_one_of(vec![TokenString,TokenStringQuote])?;
        let content=if string.token_type!=TokenStringQuote { string.content.to_string() } else { String::from("") };

        let value=Value::ObjString(content); // copies out  
        chunk.write_constant(value, string.line);

        if string.token_type!=TokenStringQuote {
            self.consume(TokenStringQuote)?;
        }
        Ok(())
    }

    // call based on enum
    fn call_parse_fn(&mut self, chunk:&mut Chunk, ty:ParseFn, can_assign:bool)->Result<()>{
        match ty {
            ParseNumber => self.number(chunk),
            ParseUnary => self.unary(chunk),
            ParseBinary => self.binary(chunk),
            ParseGrouping => self.grouping(chunk),
            ParseString => self.string(chunk),
            ParseIdent => self.parse_ident(chunk, can_assign),
        }
    }

    fn parse_precedence(&mut self, chunk: &mut Chunk, prec:Precedence)->Result<()> {
        self.advance()?;
        // get rule based on parser.prev.type
        let prev=self.expect_prev()?;

        // no rule exists , then prefix or not
        // let rule=self.expect_rule(prev)?;
        let rule=ParseRule::get_rule(prev.token_type);

        // we should first have a prefix (expect prefix)
        let prefix=rule.prefix;
        if prefix.is_none() {
            let msg=format!("Expected expression but got: '{}'", prev.content);
            return self.report_msg(prev, msg);
        }

        let prefix_fn=prefix.unwrap();

        let can_assign=prec.get_precedence_val() <= PrecAssign.get_precedence_val();

        self.call_parse_fn(chunk, prefix_fn, can_assign)?;

        // infix down here - pratt parsing
        loop {
            if self.is_done() {
                break;
            }

            let curr_tok=self.expect_current()?;
            // let rule=self.expect_rule(curr_tok)?;
            let rule=ParseRule::get_rule(curr_tok.token_type);
            
            // when rule.prec is PrecNone this will break - RightParen breaks before advance
            if prec.get_precedence_val() > rule.prec.get_precedence_val() {
                break;
            }

            // break if curr_tok no infix?

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
            self.call_parse_fn(chunk, infix, can_assign)?;
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
            let res=self.delim_scanner.advance(tok.token_type);

            if let Err(delim_err) = res {
                self.report_msg(tok, delim_err)?;
            }

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
    
    /// Match token type against curr_tok: return false if not the same, else advance and return true
    fn match_token(&mut self, ty:TokenType)->bool {
        match self.curr_tok {
            Some(tok) => {
                if !(tok.token_type==ty) {
                    return false;
                }
                self.advance();
                return true;
            },
            None => false
        }
    }

    // EOF is implicit so consume means we expect some actual token type
    /// ty is the expected token to match curr_tok
    fn consume(&mut self, ty:TokenType)->Result<Token<'src>>{
        let type_string=ty.get_repr();
        if let Some(tok) = self.curr_tok {
            if tok.token_type.eq(&ty) {
                self.advance()?;
                
                if ty.eq(&TokenSemiColon) {
                    self.is_stmt=true;
                }

                Ok(tok)
            } else {
                let msg=format!("Expected {} but got {}", type_string, tok.content);
                Err(self.report_msg(tok, &msg).unwrap_err())
            }
        } else {
            let msg=format!("Expected {} but got end of input.", type_string);
            Err(self.report_err(&msg).unwrap_err())
        }
    }

    fn consume_one_of(&mut self, ty:Vec<TokenType>)->Result<Token<'src>>{
        let type_string:Vec<String>=ty.iter().map(|x| format!("'{}'", x.get_repr())).collect();
        let type_string=type_string.join(", ");

        if let Some(tok) = self.curr_tok {
            if ty.contains(&tok.token_type) {
                self.advance()?;
                Ok(tok)
            } else {
                let msg=format!("Expected one of {} but got {}", type_string, tok.content);
                Err(self.report_msg(tok, &msg).unwrap_err())
            }
        } else {
            let msg=format!("Expected one of {} but got end of input.", type_string);
            Err(self.report_err(&msg).unwrap_err())
        }
    }

    /// add string to constants and return index in constants
    fn add_string(&mut self, chunk: &mut Chunk, ident:Token<'src>)->usize {
        let string=Value::ObjString(ident.content.to_string());
        chunk.add_constant(string, ident.line)
    }

    /// Consume identifier, equals and emit OP_SET_GLOBAL
    fn parse_variable_assignment(&mut self, chunk: &mut Chunk)->Result<()> {
        let ident=self.consume(TokenIdent)?;

        self.consume(TokenEqual)?;
        self.expression(chunk)?;

        // hash the contents not the ptr
        let mut ident_hash=ident.hash_content();

        chunk.write_op(OpSetGlobal(ident_hash), ident.line);

        Ok(())
    }

    /// Parse get for an identifier - prefix func for TokenIdent
    fn parse_ident(&mut self, chunk: &mut Chunk, can_assign:bool)->Result<()> {
        // get identifier
        let ident=self.expect_prev()?;
        self.expect_token_type(ident, TokenIdent, "identifier")?;

        // use hash to get value instead of full string (less work at runtime)
        let ident_hash=ident.hash_content();

        // log::debug!("match equals:{}", self.match_token(TokenEqual));

        // write op here
        if self.match_token(TokenEqual) {
            println!("here");
            if !can_assign {
                let msg=format!("Can't assign to {}", ident.content);
                self.report_msg(ident, msg)?;
            }

            self.expression(chunk)?;
            chunk.write_op(OpSetGlobal(ident_hash), ident.line);
            self.consume(TokenSemiColon)?;

        } else {    
            chunk.write_op(OpGetGlobal(ident_hash), ident.line);

        }
        Ok(())
    }

    /// Grammar functions
    
    // let x=2;
    fn let_declaration(&mut self, chunk: &mut Chunk)->Result<()>  {
        self.parse_variable_assignment(chunk)?;
        self.consume(TokenSemiColon)?;
        Ok(())
    }

    // does (expression | statement)
    /// Return true if expression was compiled, else false
    fn declaration(&mut self, chunk: &mut Chunk)->Result<bool>  {
        // Put statement types here
        if self.match_token(TokenLet) {
            self.let_declaration(chunk)?;
            Ok(false)
        } else {
            self.expression(chunk)?;
            Ok(true)
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk)->Result<()> {
        // at first: only exprs

        self.advance()?;

        while let Some(_) = self.curr_tok {
            let res=self.declaration(chunk)?;
        }

        log::debug!("After finishing: is_stmt {}", self.is_stmt);

        // return value for expr
        if !self.is_stmt {
            self.end_compile(chunk);
        }

        match self.delim_scanner.end() {
            Err(delim_err) => self.report_err(delim_err),
            _ => Ok(())
        }

        // consume(EOF, expect end of expr)
        // Ok(())
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
        let reported_msg=format!("[line {}] Error", token.line);

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

    compile:
        while has next():
            declaration()
    
    declaration()
        -> if match(let) -> let_declaration()
        -> else: statement()
    
    statement(): -> statements must have a stack effect of zero (leave the stack unchanged after done)
        -> check current token type -> if print, or a func call => make call
        -> else: exprStmt (calls expression, consumes semicolon, then pops from stack) 
    
    to make everything an expression: dont use statements except for var assignment?
        - let x = 2; statement
        - (let x = 2) - expr
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
fn test_debug() {
    let mut p=Parser::new("x+y=3;");
    let mut chunk=Chunk::new();

    let res=p.compile(&mut chunk);
    dbg!(chunk);
}




/*
We map each token type to a different kind of expression. We define a function for each expression that outputs the appropriate bytecode. 
Then we build an array of function pointers. The indexes in the array correspond to the TokenType enum values,
and the function at each index is the code to compile an expression of that token type.


*/