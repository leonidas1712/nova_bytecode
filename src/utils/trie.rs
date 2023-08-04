// trie to store TokenType indexed by &str (char)
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::vec;
use std::fmt::Display;

use crate::scanner::tokens::{TokenType};
use crate::utils::constants::*;
use crate::scanner::tokens::*;


#[derive(Debug)]
pub struct TrieNode<V> where V:Hash {
    pub char:char,
    pub children:HashMap<char,TrieNode<V>>,
    pub value:Option<V> // mark node as terminal with value
}
impl<V> TrieNode<V> where V:Hash + Clone {
    pub fn new(char:char)->TrieNode<V> {
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

    pub fn get_child(&self, char:char)->Option<&TrieNode<V>> {
        self.children.get(&char)
    }
    pub fn get_child_mut(&mut self, char:char)->Option<&mut TrieNode<V>> {
        self.children.get_mut(&char)
    }

    pub fn get_value(&self)->Option<V> {
        self.value.clone()
    }

    pub fn set_value(&mut self, ty:V) {
        self.value.replace(ty);
    }

    pub fn empty()->TrieNode<V> {
        TrieNode::new(SPACE)
    }

    pub fn is_empty(&self)->bool {
        self.char==SPACE
    }
}

// new, add_key(key:&str, ty:TokenType), get_type(key:&str)->Option<TokenType>
pub struct Trie<K,V> where V:Hash + Display{
    pub root:TrieNode<V>,
    pub reverse_map:HashMap<V,K> // TokenType -> &str
}

// K=&str, V=TokenType 
impl<K,V> Trie<K,V> where K:ToString + Hash, V:Hash + Display + Clone + Eq {
    pub fn new()->Trie<K,V> {
        Trie { root: TrieNode::empty(), reverse_map:HashMap::new() }
    }

    pub fn add_key(&mut self, key:K, ty:V) {
        let to_string=key.to_string();
        let mut chars=to_string.chars().peekable();
        let mut node=&mut self.root;

        while let Some(char) = chars.next() {
            if !node.has_child(char) {
                node.add_child(char);
            }
            node=node.get_child_mut(char).unwrap();
        }

        self.reverse_map.insert(ty.clone(), key);
        node.set_value(ty);
    }

    pub fn get_type(&self, key:K)->Option<V> where K:ToString{
        let key=key.to_string();
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
    pub fn get_all_from_node(node:&TrieNode<V>, stack:&mut Vec<char>)->Vec<String>{
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

    // get_all in trie
    pub fn get_all(&self)->Vec<String>{
        let mut st:Vec<char>=vec![];
        Self::get_all_from_node(&self.root, &mut st)
    }

    /// (TokenType -> &str)
    /// Given value, return key associated with the value (reverse mapping)
    pub fn get_key_from_value(&self, value:V)->Option<&K> {
        self.reverse_map.get(&value)
    }
}

// advance until result reached or keyword invalidated
// advance until the last match



#[test]
fn trie_test() {
    let mut t:Trie<&'static str, TokenType>=Trie::new();
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

    let mut t:Trie<&'static str, TokenType>=Trie::new();
    
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