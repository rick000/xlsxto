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

#[derive(Debug)]
pub struct Len {
    field_type: String,
    len_min: i32,
    len_max: i32,
}

impl Len {
    pub fn new(field_type: &String) -> Len {
        Len {
            field_type: field_type.clone(),
            len_min: -1,
            len_max: -1,
        }
    }
}

impl Checker for Len {
    fn test(&self, value: &String) -> bool {
        if self.field_type.contains("string") || self.field_type.contains("localize") {
            let l = value.len();
            return l as i32 >= self.len_min && l as i32 <= self.len_max;
        }
        if self.field_type.contains("[]") {
            let a = value.split(",").enumerate().count() as i32;
            return a >= self.len_min && a <= self.len_max;
        }

        false
    }

    fn add(&mut self, _: Box<dyn Checker>) -> bool {
        false
    }

    fn expect_more(&self) -> bool {
        if self.len_min < 0 || self.len_max < 0 {
            return true;
        }
        false
    }

    fn add_param(&mut self, param: String) -> bool {
        if self.len_min < 0 {
            self.len_min = param.parse::<i32>().unwrap();
            return true;
        }
        if self.len_max < 0 {
            self.len_max = param.parse::<i32>().unwrap();
            return true;
        }

        return false;
    }

    fn need_full_load(&self) -> bool {
        false
    }
}
