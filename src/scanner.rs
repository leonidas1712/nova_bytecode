use crate::utils::constants::*;

pub mod tokens;
pub mod delim;

use tokens::*;

#[derive(Debug)]
pub struct Scanner<'src> {
    source:&'src str,
    chars:LookaheadChars<'src>,
    start:usize, // index in source for start of curr lexeme
    current:usize, // index of current char
    line:usize, // line_num,
    is_string:bool
}

// valid char for ident: alphanumeric or '_'
fn is_valid_ident_char(char:char)->bool {
    char.is_alphanumeric() || char==UNDERSCORE
}

impl<'src> Scanner<'src> {
    pub fn new<'source>(source:&'source str)->Scanner<'source>{
        let chars=LookaheadChars::new(source);
        Scanner { source, chars, start: 0, current: 0, line: 1, is_string:false }
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
        Token { token_type, content, line:self.line }
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

        self.make_token(if float { TokenFloat } else { TokenInteger })
    }

    // when current char is open_string
    // is_string F: emit quote, set to true
    // is string T: advance while, emit TokenString, end

    // if char is not OPEN_STRING this method only called when string=true
    fn string(&mut self, char:char)->Token<'src> {
        if char==OPEN_STRING {
            if self.is_string {
                self.advance();
            }

            self.is_string=!self.is_string;
            return self.make_token(TokenStringQuote);
        }

        // self.start=self.current; // idx already at first char of string
        self.advance_while(|ch| ch!=OPEN_STRING);

        // let tok=self.make_token(TokenString); // start=idx of first char, curr=idx of terminator

        let content=&self.source[self.start..self.current];
        self.start=self.current;
        let tok=Token { token_type:TokenString, content, line:self.line };

        // self.advance(); // move past terminator
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

    // xyz_123
    fn identifier(&mut self)->Token<'src> {
        self.advance_while(|ch| is_valid_ident_char(ch));
        self.make_token(TokenIdent)
    }

    // no match: token ident, else use the last match possible
    // e.g '==' => last match is TokenEqEq
    // advance the scanner
    fn check_trie(&mut self, char:char)->Token<'src>{
        let mut curr_node=&KEYWORDS_TRIE.root;

        let node=curr_node.get_child(char);

        if let None = node {
            // ident handle here - loop while isdigit is alpha etc
            return self.identifier();
        }

        curr_node=node.unwrap();

        let mut length=1;

        while let Some(pk) = self.peek() {
            let try_get=curr_node.get_child(pk);
            if let Some(child) = try_get {
                curr_node=child;
                self.advance();
                length+=1;
            } else {
                break;
            }
        }
        // if single char token: immediately return the type (dont filter out)

        // true: dont use as identifier (use value from trie for type)
        // false: use as identifier 
            // e.g 'if': use value we got
            // 'if3': var name

        let mut use_trie_value=|| {
            // first char not alphanum or _
            if !is_valid_ident_char(char) {
                return true;
            }

            // e.g 'ifr': use as ident
            if let Some(pk) = self.peek() {
                if is_valid_ident_char(pk) {
                    return false;
                } 
            }

            true
        };

        let get=curr_node.get_value()
        .filter(|_| use_trie_value())
        .map(|ty| self.make_token(ty));

        if let Some(tok) = get {
            tok
        } else {
            self.identifier()
        }
    }

    // collect consumed into String repr for debugging
    fn serialize(self)->String {
        let s=self.into_iter().map(|tok| tok.to_string()).collect::<Vec<String>>().join(",");
        format!("[{s}]")
    }
}


// "( poljroj"

impl<'src> Iterator for Scanner<'src> {
    type Item = Token<'src>;
    // advance: self.current+=1;
    // make_token: self.start=self.current

    // scan_token
    fn next(&mut self) -> Option<Self::Item> {
        let peek=self.peek();

        if self.is_string {
            if peek.is_none() {
                return None;
            }
            return Some(self.string(peek.unwrap()));
        }


        self.skip_whitespace();

        self.start=self.current;

        if self.start >= self.source.len() {
            return None;
        }

        let nxt=self.advance();

        
        match nxt {
            Some(char) => {
                match char {
                    // char if self.is_string => Some(self.string(char)),
                    char if self.is_string || char==OPEN_STRING => Some(self.string(char)), 
                    char if char.is_ascii_digit() => Some(self.number()),
                    _ => Some(self.check_trie(char))
                }
            },

            None => Some(self.make_token(TokenError)) // err since OOB for start already checked
        }
    }
}

#[test]
fn test_scanner() {
    let inp="(2345) 23";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenLeftParen('('),TokenInteger('2345'),TokenRightParen(')'),TokenInteger('23')]");

    
    let inp="  30   40 \n 50   \t 60 \r   700.30  ";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('30'),TokenInteger('40'),TokenInteger('50'),TokenInteger('60'),TokenFloat('700.30')]");

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
    assert_eq!(s.serialize(), "[TokenInteger('4'),TokenEqEq('=='),TokenInteger('2'),TokenEqual('='),TokenInteger('3'),TokenEqEq('=='),TokenInteger('5')]");

    let inp="!4 != !0 != 5";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenNot('!'),TokenInteger('4'),TokenNotEq('!='),TokenNot('!'),TokenInteger('0'),TokenNotEq('!='),TokenInteger('5')]");

    let inp="4 < 5 <= 6 > 5 >= 10 < 9 >=2>8";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('4'),TokenLess('<'),TokenInteger('5'),TokenLessEq('<='),TokenInteger('6'),TokenGt('>'),TokenInteger('5'),TokenGtEq('>='),TokenInteger('10'),TokenLess('<'),TokenInteger('9'),TokenGtEq('>='),TokenInteger('2'),TokenGt('>'),TokenInteger('8')]");
}

#[test]
fn test_comment() {
    let inp="2/3";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('2'),TokenSlash('/'),TokenInteger('3')]");

    let inp="\n\t 2/3 // c1 \n 40 // c2\n  400 // \t another comment \n 50 ";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('2'),TokenSlash('/'),TokenInteger('3'),TokenInteger('40'),TokenInteger('400'),TokenInteger('50')]");
}

// make this TokenSingleQuote, String, TokenSingleQuote
#[test]
fn test_string() {
    // let inp="245  \"  \n        some     \"   23";
    // // TokenString("some string..")
    // let mut s=Scanner::new(inp);
    // dbg!(s.serialize());
    // // assert_eq!(s.serialize(),  "[TokenInteger('2'),TokenString(' some string lit '),TokenInteger('3')]");

    // let inp="\"string 3irjij";
    // let mut s=Scanner::new(inp);
    // dbg!(s.serialize());

    let inp="234 \"    som   \" 454";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('234'),TokenStringQuote('\"'),TokenString('    som   '),TokenStringQuote('\"'),TokenInteger('454')]");

    let inp="(2+2) \"  (2+3+4)   \n\t   5+6 \"  234 + 56";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),  "[TokenLeftParen('('),TokenInteger('2'),TokenPlus('+'),TokenInteger('2'),TokenRightParen(')'),TokenStringQuote('\"'),TokenString('  (2+3+4)   \n\t   5+6 '),TokenStringQuote('\"'),TokenInteger('234'),TokenPlus('+'),TokenInteger('56')]");

    let inp="\" \", 23, \" \"";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),  "[TokenStringQuote('\"'),TokenString(' '),TokenStringQuote('\"'),TokenComma(','),TokenInteger('23'),TokenComma(','),TokenStringQuote('\"'),TokenString(' '),TokenStringQuote('\"')]");

}

#[test]
fn test_scanner_trie() {
    let inp="\n(23) 1+2-3/4*5;\n c,z 2.0";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenLeftParen('('),TokenInteger('23'),TokenRightParen(')'),TokenInteger('1'),TokenPlus('+'),TokenInteger('2'),TokenMinus('-'),TokenInteger('3'),TokenSlash('/'),TokenInteger('4'),TokenStar('*'),TokenInteger('5'),TokenSemiColon(';'),TokenIdent('c'),TokenComma(','),TokenIdent('z'),TokenFloat('2.0')]");

    let inp="\n1=2 3==4 0!=1 5<2 3<=4 3>4, 3>=4";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(), "[TokenInteger('1'),TokenEqual('='),TokenInteger('2'),TokenInteger('3'),TokenEqEq('=='),TokenInteger('4'),TokenInteger('0'),TokenNotEq('!='),TokenInteger('1'),TokenInteger('5'),TokenLess('<'),TokenInteger('2'),TokenInteger('3'),TokenLessEq('<='),TokenInteger('4'),TokenInteger('3'),TokenGt('>'),TokenInteger('4'),TokenComma(','),TokenInteger('3'),TokenGtEq('>='),TokenInteger('4')]");
}
// 1.first char no match with trie e.g xyz
// 2. matches but doesnt complete e.g ifxy
// 3. matches and next is non ident char e.g if 123
// 4. matches, next is ident char, => !x
// 5. mix: !x if false iffalser iffalse

#[test]
fn test_ident() {
    let inp="\txyz_123\n";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),"[TokenIdent('xyz_123')]");

    let inp="ifxy";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),"[TokenIdent('ifxy')]");

    let inp="if xy";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),"[TokenIf('if'),TokenIdent('xy')]");

    let inp="if !ifr";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),  "[TokenIf('if'),TokenNot('!'),TokenIdent('ifr')]");

    let inp="falsefun fun funfalse for!x";
    let mut s=Scanner::new(inp);
    assert_eq!(s.serialize(),"[TokenIdent('falsefun'),TokenFunc('fun'),TokenIdent('funfalse'),TokenIdent('for'),TokenNot('!'),TokenIdent('x')]");
}

#[test]
pub fn test_many() {
    let code = "fun func_name( if_m, fun_c, func_d) {\n\tlet x = 200.35;\n\tx\n}";
    let mut s=Scanner::new(code);
    assert_eq!(s.serialize(), "[TokenFunc('fun'),TokenIdent('func_name'),TokenLeftParen('('),TokenIdent('if_m'),TokenComma(','),TokenIdent('fun_c'),TokenComma(','),TokenIdent('func_d'),TokenRightParen(')'),TokenLeftBrace('{'),TokenLet('let'),TokenIdent('x'),TokenEqual('='),TokenFloat('200.35'),TokenSemiColon(';'),TokenIdent('x'),TokenRightBrace('}')]");

    let code = "fun func_name(ab, cd, if_m, falser) {\n\tif (x < 2) { ab + cd / m }\n\telse { 90 + falser }\n}";
    let mut s=Scanner::new(code);
    assert_eq!(s.serialize(), "[TokenFunc('fun'),TokenIdent('func_name'),TokenLeftParen('('),TokenIdent('ab'),TokenComma(','),TokenIdent('cd'),TokenComma(','),TokenIdent('if_m'),TokenComma(','),TokenIdent('falser'),TokenRightParen(')'),TokenLeftBrace('{'),TokenIf('if'),TokenLeftParen('('),TokenIdent('x'),TokenLess('<'),TokenInteger('2'),TokenRightParen(')'),TokenLeftBrace('{'),TokenIdent('ab'),TokenPlus('+'),TokenIdent('cd'),TokenSlash('/'),TokenIdent('m'),TokenRightBrace('}'),TokenElse('else'),TokenLeftBrace('{'),TokenInteger('90'),TokenPlus('+'),TokenIdent('falser'),TokenRightBrace('}'),TokenRightBrace('}')]");

    let code="(x $ y + map >> succ)";
    let mut s=Scanner::new(code);
    assert_eq!(s.serialize(), "[TokenLeftParen('('),TokenIdent('x'),TokenInfix('$'),TokenIdent('y'),TokenPlus('+'),TokenIdent('map'),TokenPipe('>>'),TokenIdent('succ'),TokenRightParen(')')]");
}

#[test]
fn test_debug() {
    let code="print(2)";
    let mut s=Scanner::new(code);
    dbg!(s.serialize());
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