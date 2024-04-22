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
use super::Checker;
use std::fmt::Debug;

pub trait TypeGreater: Debug {
    fn check_condition(&self, field_type: &String, base: &String, value: &String) -> bool;
}

#[derive(Debug)]
pub struct Generic<T: TypeGreater + Default> {
    field_type: String,
    base: String,
    greater: T,
}

impl<T> Generic<T>
where
    T: TypeGreater + Default,
{
    pub fn new(f: &String) -> Self {
        Generic {
            field_type: f.clone(),
            base: String::from(""),
            greater: T::default(),
        }
    }
}

impl<T> Checker for Generic<T>
where
    T: TypeGreater + Default,
{
    fn test(&self, value: &String) -> bool {
        if value.trim().is_empty() {
            return false;
        }

        return self
            .greater
            .check_condition(&self.field_type, &self.base, value);
    }

    fn add(&mut self, _: Box<dyn Checker>) -> bool {
        false
    }

    fn expect_more(&self) -> bool {
        if self.base.is_empty() {
            return true;
        }
        false
    }

    fn add_param(&mut self, param: String) -> bool {
        if self.base.is_empty() {
            self.base = param;
            return true;
        }

        false
    }

    fn need_full_load(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct _GT {}

#[derive(Debug, Default)]
pub struct _GE {}
pub type Gt = Generic<_GT>;
pub type Ge = Generic<_GE>;

impl TypeGreater for _GT {
    fn check_condition(&self, field_type: &String, base: &String, value: &String) -> bool {
        if field_type.contains("float") {
            let base = base.parse::<f32>().unwrap();
            let v = value.parse::<f32>().unwrap();
            return base < v;
        }

        if field_type.contains("int") {
            let base = base.parse::<i32>().unwrap();
            let v = value.parse::<i32>().unwrap();
            return base < v;
        }
        false
    }
}

impl TypeGreater for _GE {
    fn check_condition(&self, field_type: &String, base: &String, value: &String) -> bool {
        if field_type.contains("float") {
            let base = base.parse::<f32>().unwrap();
            let v = value.parse::<f32>().unwrap();
            return base <= v;
        }

        if field_type.contains("int") {
            let base = base.parse::<i32>().unwrap();
            let v = value.parse::<i32>().unwrap();
            return base <= v;
        }
        false
    }
}
