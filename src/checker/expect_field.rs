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
use crate::ALLXLSX;

#[derive(Debug)]
pub struct ExpectField {
    tab: String,
    field: String,
}

impl ExpectField {
    pub fn new() -> ExpectField {
        ExpectField {
            tab: String::from(""),
            field: String::from(""),
        }
    }
}

impl Checker for ExpectField {
    fn test(&self, _value: &String) -> bool {
        return true;
    }

    fn add(&mut self, _: Box<dyn Checker>) -> bool {
        false
    }

    fn expect_more(&self) -> bool {
        if self.tab.is_empty() || self.field.is_empty() {
            return true;
        }

        false
    }

    fn add_param(&mut self, param: String) -> bool {
        if self.tab.is_empty() {
            self.tab = param.replace("'", "");
            return true;
        }

        if self.field.is_empty() {
            self.field = param.replace("'", "");
            return true;
        }

        false
    }

    fn need_full_load(&self) -> bool {
        true
    }

    fn test_on_all_load(&self, value: &String, all: &ALLXLSX) -> bool {
        if self.tab.is_empty() || self.field.is_empty() {
            return true;
        }

        return all.has_field(&self.tab, &self.field, value);
    }
}
