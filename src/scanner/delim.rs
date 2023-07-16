use crate::utils::constants::*;
use super::tokens::*;

// DelimErr -> pair of expected token and got (got can be None for unterm)
// 2) -> unmatched ')'
// (2 => unmatched '('

// Delim: pair(opentype, closetype) + ignore:bool
    // ignore means when this opener is at front of stack, dont push any other openers onto stack
    // e.g "([" -> when " on stack, we dnt push ([
    // but closer always pops from stack

// Err report: just return TokenDelimErr(string) => use string to report

// DelimScanner::new(Vec<Delim<) => initialise
    // Delim: TokenType open, TokenType close, ignore:bool
    // init Stack<TokenType> + openers:Vec<(TokenType, bool:ignore)>, closers:Vec<TokenType> => corresponding opener/closer at same idx in arrays

// advance(ty:TokenType)->Result<(),String> => Err(string) if err, else ok
   // if ty is an opener and stack[-1] is not ignore:
        // else: stack.push(ty)

    // else ty is a closer:
        // if ty closes stack[-1]: stack.pop() -> err if empty stack (unmatched {closer})
        // else ty doesn't close stack[-1] and stack[-1] is not ignore: -> err if empty 
            // err: ty closer doesn't match stack[-1] opener
    // if ty not in open/close => ok

#[derive(Debug,Clone,Copy)]
pub struct Delimiter {
    pub opener:TokenType,
    pub closer:TokenType,
    pub can_ignore:bool // when opener is on stack[-1] we ignore other openers e.g \"
}

impl Delimiter {
    pub fn new(opener:TokenType, closer:TokenType, can_ignore:bool)->Delimiter {
        Delimiter {
            opener,
            closer,
            can_ignore
        }
    }

    pub fn opened_by(&self, ty:TokenType)->bool {
        self.opener.eq(&ty)
    }

    pub fn closed_by(&self, ty:TokenType)->bool {
        self.closer.eq(&ty)
    }
}

#[derive(Debug,Clone)]
pub struct DelimiterScanner {
    delims:Vec<Delimiter>, // look through this array to check if ty is a opener/closer or none
    stack:Vec<Delimiter>, // opener on here
    err:Option<String> // err msg 
}

impl DelimiterScanner {
    pub fn new(delims: Vec<Delimiter>)->DelimiterScanner {
        DelimiterScanner { delims: delims, stack: vec![], err:None }
    }

    /// Get delimiter corresponding to ty as opener or None if DNE
    pub fn get_opener(&self, ty:TokenType)->Option<&Delimiter> {
        self.delims.iter()
        .filter(|d| d.opened_by(ty))
        .last()
    }

    /// Get delimiter corresponding to ty as closer or None if DNE
    pub fn get_closer(&self, ty:TokenType)->Option<&Delimiter> {
        self.delims.iter()
        .filter(|d| d.closed_by(ty))
        .last()
    }

    // return true if stack[-1] is ignore, else false (false for empty)
    pub fn is_curr_ignore(&self)->bool {
        match self.stack.last() {
            Some(val) => val.can_ignore,
            _ => false
        }
    }

    // 2)
    /// True if ty closes stack[-1] opener - return None if stack empty
    pub fn closes_current(&self, ty:TokenType)->Option<bool> {
        match self.stack.last() {
            Some(opener) => {
                // stack should only have openers: panic if not true
                Some(opener.closed_by(ty))
            },
            None => None
        }
    }
    // (stack empty or stack[-1] is not ignore)
    fn can_push_opener(&self)->bool {
        match self.stack.last() {
            Some(opener) => {
                !opener.can_ignore // push if !ignore
            },
            None => true
        }
    }

    pub fn advance(&mut self, ty:TokenType)->std::result::Result<(),String> {
        // advance(ty:TokenType)->Result<(),String> => Err(string) if err, else ok
    // if ty is an opener and (stack empty or stack[-1] is not ignore):
        // push ty

    // else ty is a closer:
        // stack empty: err => unmatched closer
        // if ty closes stack[-1]: stack.pop(), return;
        // else ty doesn't close stack[-1] and stack[-1] is not ignore:
            // err: ty closer doesn't match stack[-1] opener
    // if ty not in open/close => ok
        
        let try_opener=self.get_opener(ty);

        if try_opener.is_some() {
            let delim=try_opener.unwrap();

            if self.can_push_opener() {
                self.stack.push(*delim);
                return Ok(())
            }
        } 
        
        let try_closer=self.get_closer(ty);

        if !try_closer.is_some() {
            return Ok(())
        }


        match self.stack.last() {
            Some(val) => {
                    // if ty closes stack[-1]: stack.pop(), return;
                // else ty doesn't close stack[-1] and stack[-1] is not ignore:
                    // err: ty closer doesn't match stack[-1] opener

                if val.closed_by(ty) {
                    self.stack.pop();
                    Ok(())
                } else if !val.can_ignore {
                    Err(format!("closing token {} does not match opening token {}", ty.get_repr(), val.opener))
                } else {
                    Ok(())
                }

            },
            None => Err(format!("unmatched closing token: {}", ty.get_repr()))
        }
    }

    // error if stack not empty 
    fn end(&self)->Result<(),String> {
        match self.stack.last() {
            Some(delim) => {
                Err(format!("unterminated opening token: {}", delim.opener.get_repr()))
            },
            None => Ok(())
        }
    }
}

#[test]
fn test_delim() {
    let d1=Delimiter::new(TokenLeftParen, TokenRightParen, false);
    let d2=Delimiter::new(TokenString, TokenString, true);
    let d3=Delimiter::new(TokenLeftBrace, TokenRightBrace, false);
    let v=vec![d1,d2,d3];

    let d=DelimiterScanner::new(v);
    assert_eq!(d.get_opener(TokenLeftParen).unwrap().closer, TokenRightParen);
    assert_eq!(d.get_closer(TokenRightParen).unwrap().opener, TokenLeftParen);

    assert_eq!(d.get_opener(TokenString).unwrap().closer, TokenString);
    assert_eq!(d.get_closer(TokenString).unwrap().opener, TokenString);

    assert!(d.closes_current(TokenLeftParen).is_none());
}

#[test]
fn test_delim_advance() {
    let d1=Delimiter::new(TokenLeftParen, TokenRightParen, false);
    let d2=Delimiter::new(TokenSingleQuote, TokenSingleQuote, true);
    let d3=Delimiter::new(TokenLeftBrace, TokenRightBrace, false);
    let v=vec![d1,d2,d3];
    let v2=v.clone();

    let mut d=DelimiterScanner::new(v);

    d.advance(TokenLeftParen);
    d.advance(TokenRightParen);
    assert!(d.end().is_ok()); // "()"

    d.advance(TokenSingleQuote);
    d.advance(TokenRightParen);

    assert!(d.end().is_err()); // ")

    d.advance(TokenSingleQuote);
    assert!(d.end().is_ok()); // ")"

    d.advance(TokenLeftParen);
    d.advance(TokenSingleQuote);
    assert!(d.end().is_err()); // ("

    d.advance(TokenSingleQuote);
    d.advance(TokenRightParen); // ("")
    assert!(d.end().is_ok());    

    d.advance(TokenLeftParen);
    let res=d.advance(TokenRightBrace);
    assert!(res.unwrap_err().contains("does not match"));
    
    let mut d2=DelimiterScanner::new(v2);
    let res=d2.advance(TokenRightParen);
    assert!(res.unwrap_err().contains("unmatched"))
}
