// trie to store TokenType indexed by &str (char)
use std::collections::{HashMap, HashSet};
use std::vec;

use crate::scanner::{TokenType, Token};
use crate::utils::constants::*;

#[derive(Debug)]
pub struct TrieNode {
    pub char:char,
    pub children:HashMap<char,TrieNode>,
    pub value:Option<TokenType> // mark node as terminal with value
}
impl TrieNode {
    pub fn new(char:char)->TrieNode {
        TrieNode {
            char,
            children:HashMap::new(),
            value:None
        }
    }

    pub fn has_child(&self, char:char)->bool {
        self.children.contains_key(&char)
    }

    // add child node if it DNE
    pub fn add_child(&mut self, char:char) {
        if !self.has_child(char) {
            let new_node=TrieNode::new(char);
            self.children.insert(char, new_node);
        }
    }  

    pub fn get_child(&self, char:char)->Option<&TrieNode> {
        self.children.get(&char)
    }
    pub fn get_child_mut(&mut self, char:char)->Option<&mut TrieNode> {
        self.children.get_mut(&char)
    }

    pub fn get_value(&self)->Option<TokenType> {
        self.value.clone()
    }

    pub fn set_value(&mut self, ty:TokenType) {
        self.value.replace(ty);
    }

    pub fn empty()->TrieNode {
        TrieNode::new(SPACE)
    }

    pub fn is_empty(&self)->bool {
        self.char==SPACE
    }
}

// new, add_key(key:&str, ty:TokenType), get_type(key:&str)->Option<TokenType>
pub struct Trie {
    root:TrieNode
}

impl Trie {
    pub fn new()->Trie {
        Trie { root: TrieNode::empty() }
    }

    pub fn add_key<K>(&mut self, key:K, ty:TokenType) 
    where K:ToString {
        let to_string=key.to_string();
        let mut chars=to_string.chars().peekable();
        let mut node=&mut self.root;

        while let Some(char) = chars.next() {
            if !node.has_child(char) {
                node.add_child(char);
            }
            node=node.get_child_mut(char).unwrap();
        }

        node.set_value(ty);
    }

    pub fn get_type(&self, key:&str)->Option<TokenType>{
        let mut chars=key.chars().peekable();
        let mut node=&self.root;

        while let Some(char) = chars.next() {
            if let Some(child) = node.get_child(char) {
                node=child;
            } else {
                return None;
            }
        }

        node.get_value()
    }

    // only for debugging: ok to do String
    pub fn get_all_from_node(node:&TrieNode, stack:&mut Vec<char>)->Vec<String>{
        let mut strings:Vec<String>=vec![];
        // terminal
        if let Some(ty) = node.get_value() {
            let name:String=stack.iter().collect();
            let ty=ty.to_string();
            strings.push(format!("{}:{}", name, ty));
        }

        for (char,child) in node.children.iter() {
            stack.push(char.to_owned());
            let mut results=Self::get_all_from_node(child, stack);
            strings.append(&mut results);
            stack.pop();
        }

        strings
    }

    pub fn get_all(&self)->Vec<String>{
        let mut st:Vec<char>=vec![];
        Self::get_all_from_node(&self.root, &mut st)
    }
}

// advance until result reached or keyword invalidated
// advance until the last match


// keywords trie
pub fn setup_keywords()->Trie{
    let mut trie=Trie::new();
    trie.add_key(OPEN_EXPR, TokenLeftParen);
    trie.add_key(CLOSE_EXPR, TokenRightParen);
    trie.add_key(STMT_END, TokenSemiColon);
    trie.add_key(COMMA, TokenComma);
    trie.add_key(DOT, TokenDot);
    trie.add_key(PLUS, TokenPlus);
    trie.add_key(MINUS, TokenMinus);
    trie.add_key(SLASH, TokenSlash);
    trie.add_key(STAR, TokenStar);

    // comp
    trie.add_key(EQ, TokenEqual);
    trie.add_key(LESS_THAN, TokenLess);
    trie.add_key(GT_THAN, TokenGt);
    trie.add_key(BANG, TokenNot);


    // two char
    trie.add_key(EQ_EQ, TokenEqEq);
    trie.add_key(NOT_EQ, TokenNotEq);
    trie.add_key(LT_EQ, TokenLessEq);
    trie.add_key(GT_EQ, TokenGtEq);

    trie

}

use TokenType::*;

#[test]
fn trie_test() {
    let mut t=Trie::new();
    assert_eq!(t.get_type(""),None);

    t.add_key("if", TokenIf);
    let f=t.get_type("if");
    assert_eq!(f, Some(TokenIf));

    t.add_key("if", TokenComma);

    let f=t.get_type("if");
    assert_eq!(f, Some(TokenComma));

    let f=t.get_type("dne");
    assert_eq!(f, None);

    let f=t.get_type("i");
    assert_eq!(f, None);
}

#[test]
fn trie_test_overlap() {
    let mut t=Trie::new();
    
    t.add_key(">", TokenGt);
    t.add_key(">>", TokenPipe);
    t.add_key(">=", TokenGtEq);
    t.add_key("/", TokenSlash);
    t.add_key("//", TokenComment);
    t.add_key("if", TokenIf);

    assert_eq!(t.get_type(">"), Some(TokenGt));
    assert_eq!(t.get_type(">>"), Some(TokenPipe));
    assert_eq!(t.get_type(">="), Some(TokenGtEq));
    assert_eq!(t.get_type("/"), Some(TokenSlash));
    assert_eq!(t.get_type("//"), Some(TokenComment));
    assert_eq!(t.get_type("x1"), None); // ident

    let st:HashSet<String>=t.get_all().into_iter().collect();
    let exp:Vec<String>=vec!["/:TokenSlash", ">=:TokenGtEq", ">:TokenGt", ">>:TokenPipe", "if:TokenIf", "//:TokenComment"].into_iter().map(|x| x.to_owned()).collect();
    let exp:HashSet<String>=exp.into_iter().collect();
    assert_eq!(st, exp);
}