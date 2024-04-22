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
pub struct Eq {
    expect: String,
}

impl Eq {
    pub fn new() -> Eq {
        Eq {
            expect: String::from(""),
        }
    }
}

impl Checker for Eq {
    fn test(&self, value: &String) -> bool {
        if self.expect.trim().eq(value.trim()) {
            return true;
        }

        false
    }

    fn add(&mut self, _: Box<dyn Checker>) -> bool {
        false
    }

    fn expect_more(&self) -> bool {
        if self.expect.is_empty() {
            return true;
        }
        false
    }

    fn add_param(&mut self, param: String) -> bool {
        if self.expect.is_empty() {
            self.expect = param;
            return true;
        }
        false
    }
    fn need_full_load(&self) -> bool {
        false
    }
}
