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
pub struct Not {
    next: Option<Box<dyn Checker>>,
}

impl Not {
    pub fn new() -> Not {
        Not { next: None }
    }
}

impl Checker for Not {
    fn test(&self, value: &String) -> bool {
        if let Some(checker) = &self.next {
            return !checker.test(value);
        }
        false
    }

    fn add(&mut self, checker: Box<dyn Checker>) -> bool {
        if let Some(_) = &self.next {
            return false;
        }

        self.next = Some(checker);
        true
    }

    fn expect_more(&self) -> bool {
        if let Some(_) = &self.next {
            return false;
        }

        true
    }

    fn add_param(&mut self, _: String) -> bool {
        false
    }

    fn need_full_load(&self) -> bool {
        false
    }
}
