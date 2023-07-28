use crate::compiler::Compiler;
use crate::scanner::delim::{Delimiter, DelimiterScanner};
use crate::scanner::{tokens::*, Scanner};
use crate::data::ops::*;
use crate::utils::err::*;

use Inst::*;

use super::rules::*;
use super::rules::ParseRule;

use log::debug;

// add stacks to track () and string ""
// open token, close token for each pair
// creates a new stack for each pair and tracks => hashmap

#[derive(Debug)]
pub struct Parser<'src> {
    compiler:Compiler,
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
        let compiler=Compiler::new();

        let delimiters:Vec<Delimiter> = vec![
            Delimiter::new(TokenLeftParen, TokenRightParen, false),
            Delimiter::new(TokenStringQuote, TokenStringQuote, true)
        ];

        let delim_scanner=DelimiterScanner::new(delimiters);

        Parser { scanner, compiler, prev_tok: None, curr_tok: None, line:1, delim_scanner, is_stmt:true }
    }

    // ParseFn: assume that the token to parse is set in self.prev
    // Helpers for each parse type

    // expect_token_type(ty)->Result<()>
    fn number(&mut self, chunk: &mut Chunk)->Result<()>{
        let prev=self.expect_prev()?;
        self.expect_token_type(prev, TokenInteger, "integer")?; // only errs when bug in parser

        // convert to number
        let value:IntType = prev.content.parse().unwrap();
        let value=Value::Number(value);

        chunk.write_constant(value, prev.line);
        Ok(())
    }

    // unary called based on rules table
    fn unary(&mut self, chunk:&mut Chunk)->Result<()>{
        let prev=self.expect_prev()?;
        // next expression result goes onto stack
        // PrecUnary higher than binary => -1+2 means - will bind 1 and prevent + from consuming
        self.parse_precedence(chunk, PrecUnary)?; 


        let op = match prev.token_type {
            TokenMinus => OpNegate,
            TokenNot => OpNot,
            _ => unreachable!()
        };

        chunk.write_op(op, prev.line);

        Ok(())
    }

    // binary called based on rules table
    fn binary(&mut self, chunk:&mut Chunk)->Result<()>{
        // debug!("Called binary, curr_tok:{:?}, prev:{:?}", &self.curr_tok, &self.prev_tok);
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

    // should return result from branch selected

    // use parse_precedence if expression
    
    fn if_expression(&mut self, chunk:&mut Chunk)->Result<()> {
        self.consume(TokenLeftParen)?;
        self.expression(chunk)?; // conditional
        self.consume(TokenRightParen)?;
 
        // write jump (incomplete)
        self.is_stmt=true;
        let if_false_idx=chunk.write_op(Inst::OpIfFalseJump(0), self.line);


        self.declaration(chunk, true)?; // to execute if true

        // emit op_jump here so that if branch skips the else
        let jmp_idx=chunk.write_op(OpJump(0), self.line);


        // self.is_stmt=true;
        // handle else
        if self.match_token(TokenElse) {
            self.is_stmt=true; // set to true so if next is expr can be parsed
            self.declaration(chunk, true)?;
        }

        debug!("Ip after else: {}", chunk.get_ip().unwrap());

        let jmp_after_if=chunk.get_ip().unwrap();
        let jmp=chunk.get_op_mut(jmp_idx).unwrap();
        match jmp {
            OpJump(k) => {
                *k=jmp_after_if;
            },
            _ => unreachable!()
        }

        // update iffalsejump to be after opjump
        let if_false_jmp=chunk.get_op_mut(if_false_idx).unwrap();
        match if_false_jmp {
            OpIfFalseJump(k) => {
                *k=jmp_idx;
            },
            _ => unreachable!()
        }

        debug!("HERE");

        
        debug!("if is_stmt at the end:{}", self.is_stmt);

        // stmt: other statements can follow this
        // because of this op return is not emitted so the last value remains on the stack
        self.is_stmt=true; 


        // // self.advance();
        // self.expression(chunk)?;

        Ok(())
    }

    // An expression must leave a value on the stack
    fn expression(&mut self, chunk:&mut Chunk)->Result<()>{
        // assign is the lowest valid precedence: other ops can bind as much as possibl
        // debug!("EXPRESSION {:?}", self);
        // Block expression
        if self.match_token(TokenLeftBrace) {
            self.begin_scope(chunk)?;
            self.block(chunk)?;
            self.end_scope(chunk)?;
            return Ok(())
        } 

        if !self.is_stmt {
            return self.report_err("Expressions not allowed immediately after another expression.")
        }

        self.parse_precedence(chunk, PrecAssign)?;

        // is_stmt if prev tok is semicolon - for "x=5;"
        let is_stmt=self.expect_prev()
            .ok()
            .map(|tok| tok.token_type.eq(&TokenSemiColon))
            .unwrap_or(false);

        debug!("Expression is_stmt:{}", is_stmt);

        self.is_stmt=is_stmt;
        Ok(())
    }

    fn grouping(&mut self, chunk:&mut Chunk)->Result<()> {
        self.expression(chunk)?;
        // self.declaration(chunk)?;
        self.consume(TokenRightParen)?;
        Ok(())
    }

    // curr should be TokenString
    // advance so that curr is right past ending quote
    // string literal
    fn string(&mut self, chunk: &mut Chunk)->Result<()> {
        let string=self.consume_one_of(vec![TokenString,TokenStringQuote])?;
        let content=if string.token_type!=TokenStringQuote { string.content.to_string() } else { String::from("") };

        // let value=Value::ObjString(content); // copies out  
        chunk.load_string(content, string.line);

        if string.token_type!=TokenStringQuote {
            self.consume(TokenStringQuote)?;
        }
        Ok(())
    }

    fn literal(&mut self, chunk: &mut Chunk)->Result<()> {
        let prev=self.expect_prev()?;

        let op = match prev.token_type {
            TokenTrue => OpTrue,
            TokenFalse => OpFalse,
            _ => unreachable!()
        };

        chunk.write_op(op, self.line);
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
            ParseLiteral => self.literal(chunk)
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

        let peek=self.scanner.peek();
       

        // set curr to none if scanner is finished
        if peek.is_none() || peek.unwrap().is_ascii_whitespace() {
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

    /// Return true if current_tok is ty else false. None if empty
    fn check(&mut self, ty:TokenType)->Option<bool> {
        self.curr_tok.map(|t| t.token_type==ty)
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

    /// Parse identifier - prefix func for TokenIdent
    /// Either get the variable or set it
    /// can_assign when previous precedence is not too high e.g !false=2; => err but x=2; ok
    /// namedVariable
    fn parse_ident(&mut self, chunk: &mut Chunk, can_assign:bool)->Result<()> {
        // get identifier
        let ident=self.expect_prev()?;
        self.expect_token_type(ident, TokenIdent, "identifier")?;

        // use hash to get value instead of full string (less work at runtime)
        let ident_content=ident.content.to_string();


        // set get_op here
        let mut get_op:Inst;

        let try_local=self.compiler.resolve_local(ident.content);

        if let Some(idx) = try_local {
            get_op=OpGetLocal(idx);
        } else {
            get_op=OpGetGlobal(ident_content.clone());
        }

        // Set var here
        if self.match_token(TokenEqual) {
            if !can_assign {
                let msg=format!("Can't assign to {}", ident.content);
                self.report_msg(ident, msg)?;
            }

            self.expression(chunk)?; // assign to expression
            self.consume(TokenSemiColon)?;

            // declareVariable() here - if global do nothing. else, add local with ident
            let local_added=self.compiler.add_local(ident.content);

            // local_added: idx where loc was added      
            let mut set_op:Inst;
            if let Some(idx) = local_added {
                set_op=OpSetLocal(idx);
            } else {
                set_op=OpSetGlobal(ident_content);
            }

            chunk.write_op(set_op, ident.line);

        // Get var here
        } else {    
            chunk.write_op(get_op, ident.line);

        }
        Ok(())
    }

    /// Grammar functions
    
    // let x=2;
    // varDeclaration
    fn let_declaration(&mut self, chunk: &mut Chunk)->Result<()>  {
        self.parse_precedence(chunk, PrecAssign)?;
        Ok(())
    }

    // New scope for Compiler
    fn begin_scope(&mut self, chunk: &mut Chunk)->Result<()> {
        self.compiler.begin_scope();
        Ok(())
    }

    // Block
    // polymorphic: behave as statement or expr based on last evaluated expr
    fn block(&mut self, chunk: &mut Chunk)->Result<()> {        
        loop {
            match self.check(TokenRightBrace) {
                // not right brace: keep going
                Some(b) if !b => {
                    self.declaration(chunk, false)?;
                },
                _ => {
                    break;
                }
            }
        }

        self.consume(TokenRightBrace)?;
        debug!("BLOCK IS_STMT:{}", self.is_stmt);
        Ok(())
    }

    // End compiler scope
    fn end_scope(&mut self, chunk: &mut Chunk)->Result<()> {
        let count=self.compiler.end_scope();

        let is_expr=!self.is_stmt;
    
        chunk.write_op(OpEndScope(count, is_expr), self.line);
        Ok(())
    }

    /// does (expression | statement)
    fn declaration(&mut self, chunk: &mut Chunk, can_end:bool)->Result<()>  {
        if let None = self.scanner.peek() {
            if can_end {
                return Ok(())
            }
        }
        // Put statement types here - switch on statement
        if self.match_token(TokenLet) {
            self.let_declaration(chunk)?;

        // unit
        } else if self.match_token(TokenPrint) {
            self.expression(chunk)?;
            chunk.write_op(OpPrint, self.line);
            self.consume(TokenSemiColon)?;
        } else if self.match_token(TokenIf) {
            self.if_expression(chunk)?;
            return Ok(())
        } else {
            self.expression(chunk)?;
        }

        // put expression and block together

        Ok(())
    }

    /// Compile input string into the provided chunk. This is the entry point to the parser.
    pub fn compile(&mut self, chunk: &mut Chunk)->Result<()> {
        // at first: only exprs

        self.advance()?;

        while let Some(_) = self.curr_tok {
            self.declaration(chunk, false)?;
        }

        debug!("After finishing: is_stmt {}", self.is_stmt);

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
        debug!("{:?}", self);
        if let Some(tok) = self.curr_tok {
            self.report_msg(tok, msg)

        } else {
            self.report_msg(Token::err(self.line), msg)
        }
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
    let mut p=Parser::new("
        if (true) {
            print(2);
        }
    ");

    // let mut p=Parser::new("2+3");
    let mut chunk=Chunk::new();

    let res=p.compile(&mut chunk);


}




/*
We map each token type to a different kind of expression. We define a function for each expression that outputs the appropriate bytecode. 
Then we build an array of function pointers. The indexes in the array correspond to the TokenType enum values,
and the function at each index is the code to compile an expression of that token type.


*/