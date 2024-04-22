/*
Copyright (c) 2024- rickhan<rick.han@yahoo.com>

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use super::xlsx::ALLXLSX;
use lex_lua::{Keyword, Lexer, Punct, Token};
use std::collections::LinkedList;
use std::fmt::Debug;
use std::io::{Error, ErrorKind};

pub trait Checker: Debug {
    fn test(&self, value: &String) -> bool;
    fn add(&mut self, checker: Box<dyn Checker>) -> bool;
    fn add_param(&mut self, param: String) -> bool;
    fn expect_more(&self) -> bool;
    fn need_full_load(&self) -> bool {
        false
    }

    fn test_on_all_load(&self, _: &String, _: &ALLXLSX) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct CheckObj {
    checkers: Vec<Box<dyn Checker>>,
}

mod and;
mod empty;
mod eq;
mod expect_field;
mod gt;
mod length;
mod lt;
mod not;
mod or;
mod range;

fn func_name_to_checker_obj(name: &str, field_type: &String) -> Option<Box<dyn Checker>> {
    match name {
        "empty" => Some(Box::new(empty::Empty::new())),
        "range" => Some(Box::new(range::Range::new(field_type))),
        "gt" => Some(Box::new(gt::Gt::new(field_type))),
        "ge" => Some(Box::new(gt::Ge::new(field_type))),
        "lt" => Some(Box::new(lt::Lt::new(field_type))),
        "le" => Some(Box::new(lt::Le::new(field_type))),
        "eq" => Some(Box::new(eq::Eq::new())),
        "len" => Some(Box::new(length::Len::new(field_type))),
        "expect" => Some(Box::new(expect_field::ExpectField::new())),
        _ => None,
    }
}

fn process_token(checker: &mut CheckObj, lex: &mut Lexer, field_type: &String) -> bool {
    let mut expect_from: LinkedList<usize> = LinkedList::new();
    for (_, token) in lex.enumerate() {
        let v: bool = match token {
            Token::Keyword(word) => match word {
                Keyword::Not => {
                    let this = Box::new(not::Not::new());
                    checker.add(this);
                    expect_from.push_back(checker.len() - 1);
                    true
                }
                Keyword::And => {
                    let mut result = true;
                    if checker.len() == 0 {
                        result = false;
                    }

                    let last = checker.pop();
                    if let Some(c) = last {
                        let mut this = Box::new(and::And::new());
                        if !this.as_mut().add(c) {
                            result = false;
                        } else {
                            checker.add(this);
                            expect_from.push_back(checker.len() - 1);
                        }
                    }

                    result
                }
                Keyword::Or => {
                    let mut result = true;
                    if checker.len() == 0 {
                        result = false;
                    }

                    let last = checker.pop();
                    if let Some(c) = last {
                        let mut this = Box::new(or::Or::new());
                        if !this.as_mut().add(c) {
                            result = false;
                        } else {
                            checker.add(this);
                            expect_from.push_back(checker.len() - 1);
                        }
                    }

                    result
                }
                _ => false,
            },
            Token::Punct(punct) => match punct {
                Punct::OpenParen | Punct::Comma => {
                    let mut result = true;
                    if expect_from.is_empty() {
                        result = false;
                    }
                    result
                }
                Punct::CloseParen => {
                    let mut result = true;
                    if expect_from.is_empty() {
                        result = false;
                    } else {
                        let index = expect_from.pop_back();
                        if let Some(i) = index {
                            if checker.is_ready(i) == false {
                                result = false;
                            } else {
                                if !checker.check_finished(&mut expect_from) {
                                    result = false;
                                }
                            }
                        }
                    }
                    result
                }
                _ => false,
            },
            Token::Name(name) => {
                let mut result = true;
                let inst = func_name_to_checker_obj(name.as_ref(), &field_type);
                if let Some(c) = inst {
                    checker.add(c);
                    expect_from.push_back(checker.len() - 1);
                } else {
                    result = false;
                }
                result
            }
            Token::Numeral(num) => {
                let mut result = false;
                if !checker.is_empty() && !expect_from.is_empty() {
                    let idx = expect_from.back();
                    if let Some(i) = idx {
                        if let Some(c) = checker.get_mut(*i) {
                            if (*c).as_mut().add_param(String::from(num)) {
                                result = true;
                            }
                        }
                    }
                }
                result
            }
            Token::LiteralString(param) => {
                let mut result = false;
                if !checker.is_empty() && !expect_from.is_empty() {
                    let idx = expect_from.back();
                    if let Some(i) = idx {
                        if let Some(c) = checker.get_mut(*i) {
                            if (*c).as_mut().add_param(format!("{}", param)) {
                                result = true;
                            }
                        }
                    }
                }
                result
            }
            _ => false,
        };
        if !v {
            return false;
        }
    }

    return checker.is_all_ready();
}

pub fn generate_checker(
    expr: String,
    field_type: String,
    fname: &String,
) -> Result<CheckObj, Box<dyn std::error::Error>> {
    let mut checker = CheckObj::new();

    let expr_trimd = expr.trim();
    if expr_trimd.is_empty() || expr_trimd == "c" || expr_trimd == "~" {
        return Ok(checker);
    }

    let mut lex = Lexer::new(expr.as_bytes());
    if !process_token(&mut checker, &mut lex, &field_type) {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            format!(
                "**导表校验失败**\n**xlsx = {}**\n  unsupport check condition!",
                fname
            ),
        )));
    }

    return Ok(checker);
}

impl CheckObj {
    pub fn new() -> Self {
        CheckObj {
            checkers: Vec::new(),
        }
    }

    pub fn test(&self, value: &String) -> bool {
        for c in &self.checkers {
            if !c.test(value) {
                return false;
            }
        }
        true
    }

    pub fn test_on_all_load(&self, value: &String, all: &ALLXLSX) -> bool {
        for c in &self.checkers {
            if !c.test_on_all_load(value, all) {
                return false;
            }
        }
        true
    }

    pub fn need_full_load(&self) -> bool {
        for c in &self.checkers {
            if c.need_full_load() {
                return true;
            }
        }

        false
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Box<dyn Checker>> {
        return self.checkers.get_mut(id);
    }

    pub fn get(&self, id: usize) -> Option<&Box<dyn Checker>> {
        return self.checkers.get(id);
    }

    pub fn add(&mut self, c: Box<dyn Checker>) {
        self.checkers.push(c);
    }

    pub fn pop(&mut self) -> Option<Box<dyn Checker>> {
        return self.checkers.pop();
    }

    pub fn len(&self) -> usize {
        return self.checkers.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.checkers.is_empty();
    }

    fn is_ready(&self, index: usize) -> bool {
        let c = self.get(index);
        if let Some(check) = c {
            return !check.expect_more();
        }

        false
    }

    fn is_all_ready(&self) -> bool {
        for c in &self.checkers {
            if c.expect_more() {
                return false;
            }
        }

        true
    }

    fn check_finished(&mut self, expects: &mut LinkedList<usize>) -> bool {
        loop {
            if expects.is_empty() {
                break;
            }

            let index = expects.back();
            if let Some(i) = index {
                if *i == self.len() {
                    break;
                }

                let last = self.checkers.pop();
                if let Some(expect_from) = self.checkers.get_mut(*i) {
                    if let Some(l) = last {
                        if !expect_from.add(l) {
                            return false;
                        }

                        if !expect_from.expect_more() {
                            expects.pop_back();
                        }
                    } else {
                        break;
                    }
                } else {
                    return false;
                }
            } else {
                break;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use crate::checker::generate_checker;

    #[test]
    fn test_gt() {
        let ret = generate_checker(
            "gt(12)".to_string(),
            "int".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"13".to_string()), true);
            assert_eq!(checker.test(&"13345463".to_string()), true);
            assert_eq!(checker.test(&"1".to_string()), false);
            assert_eq!(checker.test(&"-1232141".to_string()), false);
            assert_eq!(checker.test(&"-0".to_string()), false);
            assert_eq!(checker.test(&"0".to_string()), false);
            assert_eq!(checker.test(&"12".to_string()), false);
        } else {
            assert!(false); // impossible
        }
    }

    #[test]
    fn test_lt() {
        let ret = generate_checker(
            "lt(12)".to_string(),
            "int".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"13".to_string()), false);
            assert_eq!(checker.test(&"13345463".to_string()), false);
            assert_eq!(checker.test(&"1".to_string()), true);
            assert_eq!(checker.test(&"-1232141".to_string()), true);
            assert_eq!(checker.test(&"-0".to_string()), true);
            assert_eq!(checker.test(&"0".to_string()), true);
            assert_eq!(checker.test(&"12".to_string()), false);
        } else {
            assert!(false); // impossible
        }
    }

    #[test]
    fn test_range() {
        let ret = generate_checker(
            "range(10, 100)".to_string(),
            "int".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"13".to_string()), true);
            assert_eq!(checker.test(&"13345463".to_string()), false);
            assert_eq!(checker.test(&"1".to_string()), false);
            assert_eq!(checker.test(&"-1232141".to_string()), false);
            assert_eq!(checker.test(&"-0".to_string()), false);
            assert_eq!(checker.test(&"0".to_string()), false);
            assert_eq!(checker.test(&"10".to_string()), true);
            assert_eq!(checker.test(&"100".to_string()), true);
        } else {
            assert!(false); // impossible
        }

        let ret = generate_checker(
            "range(10, 100)".to_string(),
            "float".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"13".to_string()), true);
            assert_eq!(checker.test(&"13345463.0".to_string()), false);
            assert_eq!(checker.test(&"1".to_string()), false);
            assert_eq!(checker.test(&"-1232141.0".to_string()), false);
            assert_eq!(checker.test(&"-0".to_string()), false);
            assert_eq!(checker.test(&"0".to_string()), false);
            assert_eq!(checker.test(&"10.0".to_string()), true);
            assert_eq!(checker.test(&"100.0".to_string()), true);
            assert_eq!(checker.test(&"9.9999".to_string()), false);
            assert_eq!(checker.test(&"10.0001".to_string()), true);
            assert_eq!(checker.test(&"99.9999".to_string()), true);
            assert_eq!(checker.test(&"100.0001".to_string()), false);
        } else {
            assert!(false); // impossible
        }
    }

    #[test]
    fn test_length() {
        let ret = generate_checker(
            "len(2, 4)".to_string(),
            "int[]".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"[13,13, 13, 13]".to_string()), true);
            assert_eq!(checker.test(&"13345463".to_string()), false);
            assert_eq!(checker.test(&"[12]".to_string()), false);
            assert_eq!(checker.test(&"[12,13]".to_string()), true);
            assert_eq!(checker.test(&"[12,13,14]".to_string()), true);
            assert_eq!(checker.test(&"[12,13,14,15,15,15,15]".to_string()), false);
        } else {
            assert!(false); // impossible
        }

        let ret = generate_checker(
            "len(2, 4)".to_string(),
            "string".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"helloworld".to_string()), false);
            assert_eq!(checker.test(&"hell".to_string()), true);
            assert_eq!(checker.test(&"h".to_string()), false);
            assert_eq!(checker.test(&"".to_string()), false);
            assert_eq!(checker.test(&"world".to_string()), false);
        } else {
            assert!(false); // impossible
        }
    }

    #[test]
    fn test_eq() {
        let ret = generate_checker(
            "eq(2)".to_string(),
            "int".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"2".to_string()), true);
            assert_eq!(checker.test(&"13345463".to_string()), false);
            assert_eq!(checker.test(&"[12]".to_string()), false);
            assert_eq!(checker.test(&"".to_string()), false);
            assert_eq!(checker.test(&"[12,13,14,15,15,15,15]".to_string()), false);
        } else {
            assert!(false); // impossible
        }
    }

    #[test]
    fn test_or() {
        let ret = generate_checker(
            "eq(2) or eq(3)".to_string(),
            "int".to_string(),
            &"test.xlsx".to_string(),
        );
        if let Ok(checker) = ret {
            assert_eq!(checker.test(&"2".to_string()), true);
            assert_eq!(checker.test(&"3".to_string()), true);
            assert_eq!(checker.test(&"2.5".to_string()), false);
            assert_eq!(checker.test(&"100".to_string()), false);
        } else {
            assert!(false);
        }
    }
}
