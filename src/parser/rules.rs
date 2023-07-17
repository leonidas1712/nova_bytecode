
use crate::scanner::tokens::*;

// ParseRule, Precedence

#[derive(Clone, Copy)]
pub enum Precedence {
    PrecNone,
    PrecAssign, // = (lowest valid)
    PrecOr,
    PrecAnd,
    PrecEq, // ==, !=
    PrecComp, // lt,gt, lte, gte
    PrecTerm, // + -
    PrecFactor, // *, /
    PrecUnary, // !, - e.g -2, !false
    PrecCall, // () -> for calling a function
    PrecPrimary // (expression), number, string, ident, etc
}

impl Precedence {
    /// Get precedence value from precedence enum
    pub fn get_precedence_val(&self)->usize {
        match self {
            PrecNone => 1,
            PrecAssign => 2,
            PrecOr => 3,
            PrecAnd => 4,
            PrecEq => 5,
            PrecComp => 6,
            PrecTerm => 7,
            PrecFactor => 8,
            PrecUnary => 9,
            PrecCall => 10,
            PrecPrimary => 11
        }
    }

    /// Given value, get corresponding precedence.
    pub fn get_preced_from_val(val:usize)->Precedence {
        match val {
            1 => PrecNone,
            2 => PrecAssign,
            3 => PrecOr,
            4 => PrecAnd,
            5 => PrecEq,
            6 => PrecComp,
            7 => PrecTerm,
            8 => PrecFactor,
            9 => PrecUnary,
            10 => PrecCall,
            11 => PrecPrimary,
            val if val > 11 => PrecPrimary,
            _ => unreachable!()
        }
    }

    /// Get next precedence - use for left associativity
    pub fn get_next_prec(&self)->Precedence {
        Precedence::get_preced_from_val(self.get_precedence_val()+1)
    }

    /// Get prev precedence - use for right associativity
    pub fn get_prev_prec(&self)->Precedence {
        Precedence::get_preced_from_val(self.get_precedence_val()-1)
    }
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

// matcher to call different functions: like function pointer
#[derive(Clone, Copy)]
pub enum ParseFn {
    ParseNumber,
    ParseUnary,
    ParseBinary,
    ParseGrouping,
    ParseString,
    ParseIdent,
    ParseLiteral // true,false
}

pub use ParseFn::*;

// parse rule: has Option prefix, Option infix (switch based on context)
// e.g minus is prefix sometimes, infix others

// prec: precedence used for infix op when recursing on the rest
#[derive(Clone, Copy)]
pub struct ParseRule {
    pub infix:Option<ParseFn>,
    pub prefix:Option<ParseFn>,
    pub prec:Precedence
}

impl ParseRule {
    pub fn new(prefix:Option<ParseFn>, infix:Option<ParseFn>, prec:Precedence)->ParseRule {
        ParseRule {
            infix,
            prefix,
            prec
        }
    }

    pub fn get_rule(ty:TokenType)->ParseRule{
        match ty {
            TokenInteger => ParseRule::new(Some(ParseNumber), None, PrecNone),
            TokenMinus => ParseRule::new(Some(ParseUnary), Some(ParseBinary), PrecTerm),
            TokenPlus => ParseRule::new(None, Some(ParseBinary), PrecTerm),
            TokenStar => ParseRule::new(None, Some(ParseBinary), PrecFactor),
            TokenSlash => ParseRule::new(None, Some(ParseBinary), PrecFactor),
            TokenLeftParen => ParseRule::new(Some(ParseGrouping), None, PrecNone),
            TokenStringQuote => ParseRule::new(Some(ParseString), None, PrecNone),
            TokenIdent => ParseRule::new(Some(ParseIdent), None, PrecNone),
            TokenTrue => ParseRule::new(Some(ParseLiteral), None, PrecNone),
            TokenFalse => ParseRule::new(Some(ParseLiteral), None, PrecNone),
            _ => ParseRule::new(None, None, PrecNone)
        }
    }
}