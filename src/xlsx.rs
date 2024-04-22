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
use calamine::{open_workbook, Reader, Xlsx};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use super::checker::{generate_checker, CheckObj};
use crate::generator::Generator;
use crate::get_project_name;

#[derive(Debug)]
pub struct XlsTabField {
    field_name: String,
    field_type: String,
    field_index: usize,
    row_index: usize,
    checker: CheckObj,
    condition: String,
    client_server: String,
    field_cn_name: String,
}

impl XlsTabField {
    pub fn new() -> XlsTabField {
        XlsTabField {
            field_name: "".to_string(),
            field_type: "".to_string(),
            field_index: 0,
            row_index: 0,
            checker: CheckObj::new(),
            condition: "".to_string(),
            client_server: "".to_string(),
            field_cn_name: "".to_string(),
        }
    }

    pub fn is_key_field(&self) -> bool {
        self.field_name.contains("KEY")
            || self.field_name.contains("Keys")
            || self.field_name.contains("KeyId")
            || self.field_name == "id"
            || self.field_name == "Id"
            || self.field_name == "ID"
    }

    pub fn is_remark_field(&self, _target: &String) -> bool {
        self.client_server == "none"
    }

    pub fn is_invalid_field(&self) -> bool {
        self.field_name.is_empty() || self.field_type.is_empty()
    }

    pub fn set_field_name(&mut self, name: String) {
        self.field_name = name;
    }

    pub fn get_field_name(&self) -> &String {
        &self.field_name
    }

    pub fn set_field_type(&mut self, t: String) {
        self.field_type = t;
    }

    pub fn get_field_type(&self) -> &String {
        &self.field_type
    }

    pub fn set_field_index(&mut self, i: usize) {
        self.field_index = i;
    }

    pub fn set_row_index(&mut self, i: usize) {
        self.row_index = i;
    }

    pub fn as_index(&self) -> u32 {
        self.field_index as u32
    }

    pub fn get_row_index(&self) -> u32 {
        self.row_index as u32
    }

    fn set_condition(&mut self, c: &String) {
        self.condition = c.clone();
    }

    pub fn get_condition(&self) -> &String {
        &self.condition
    }

    pub fn set_client_or_server(&mut self, c: &String) {
        self.client_server = c.clone();
    }

    pub fn set_checkers(&mut self, checkers: CheckObj) {
        self.checker = checkers;
    }

    pub fn is_valid(&self, value: &String) -> bool {
        return self.checker.test(value);
    }

    pub fn need_full_load(&self) -> bool {
        return self.checker.need_full_load();
    }

    pub fn is_valid_on_all_load(&self, value: &String, all: &ALLXLSX) -> bool {
        return self.checker.test_on_all_load(value, all);
    }

    pub fn set_field_cn_name(&mut self, name: &String) {
        self.field_cn_name = name.clone();
    }
}

pub struct XLSX {
    fields: Vec<XlsTabField>,
    values: Vec<Vec<String>>,
}

impl XLSX {
    pub fn new() -> XLSX {
        XLSX {
            fields: vec![],
            values: vec![],
        }
    }

    pub fn add_field(&mut self, field: XlsTabField) {
        self.fields.push(field);
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.values.push(row);
    }

    pub fn field_num(&self) -> usize {
        self.fields.len()
    }

    pub fn fields_list(&self) -> &Vec<XlsTabField> {
        &self.fields
    }

    pub fn value_list(&self) -> &Vec<Vec<String>> {
        &self.values
    }

    pub fn parse_from_file(
        &mut self,
        path: &String,
        fname: &str,
        is_special_xlsx: bool,
        target: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let full_name = format!("{}/{}", path, fname);
        let mut workbook: Xlsx<_> = open_workbook(full_name)?;
        for sheet in workbook.worksheets() {
            log::trace!("start parsing filename={} sheet name={}", fname, sheet.0);
            let cells = sheet.1.get_size();
            let mut i: usize = 0;
            if self.field_num() == 0 {
                let mut field_index: usize = 0;
                while i < cells.1 {
                    let mut one_field = XlsTabField::new();
                    if let Some(field_name) = sheet.1.get_value((1, i as u32)) {
                        one_field.set_field_name(field_name.to_string());
                    }
                    if is_special_xlsx {
                        one_field.set_field_type("string".to_string());
                    } else if let Some(type_name) = sheet.1.get_value((4, i as u32)) {
                        one_field.set_field_type(type_name.to_string());
                        if let Some(expr) = sheet.1.get_value((2, i as u32)) {
                            one_field.set_condition(&expr.to_string());
                            let checkers = generate_checker(
                                expr.to_string(),
                                type_name.to_string(),
                                &fname.to_string(),
                            );
                            if let Ok(css) = checkers {
                                one_field.set_checkers(css);
                            }
                        }
                        if let Some(client_or_server) = sheet.1.get_value((3, i as u32)) {
                            one_field.set_client_or_server(&client_or_server.to_string());
                        }

                        if let Some(cn_name) = sheet.1.get_value((0, i as u32)) {
                            one_field.set_field_cn_name(&cn_name.to_string());
                        }
                    }

                    one_field.set_field_index(field_index);
                    one_field.set_row_index(i);
                    if !one_field.is_invalid_field() && !one_field.is_remark_field(target) {
                        self.add_field(one_field);
                        field_index += 1;
                    }

                    i += 1;
                }
            }

            i = 5; // content start from 5th row!
            if is_special_xlsx {
                i = 1;
            }

            while i < cells.0 {
                let mut row_data: Vec<String> = vec![];
                for field in self.fields_list() {
                    if let Some(value) = sheet.1.get_value((i as u32, field.get_row_index())) {
                        row_data.push(value.to_string());
                    } else {
                        row_data.push(String::from(""));
                    }
                }
                i += 1;
                self.add_row(row_data);
            }
        }
        Ok(())
    }
}

pub struct ALLXLSX {
    all: HashMap<String, XLSX>,
}

impl ALLXLSX {
    pub fn new() -> ALLXLSX {
        ALLXLSX {
            all: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, file: XLSX) {
        self.all.insert(String::from(name), file);
    }

    pub fn gen<'a, T: Generator<'a>>(
        &'a self,
        output: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (name, xlsx) in self.all.iter() {
            let mut generator = T::new(xlsx);
            let r = generator.generate(name, output, &self);
            if let Err(e) = r {
                let notify_error = super::notify_error_info(&e);
                if let Err(s) = notify_error {
                    log::error!("notify error occurred! {}", s);
                }
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn check_xlsx_valid(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (name, xlsx) in self.all.iter() {
            let mut i = 0usize;
            let field_num = xlsx.field_num();
            while i < field_num {
                if let Some(field) = xlsx.fields.get(i) {
                    if field.need_full_load() {
                        let mut l = 4usize;
                        for value in &xlsx.values {
                            l += 1;
                            if let Some(v) = value.get(i) {
                                if field.is_valid_on_all_load(v, self) {
                                    continue;
                                }
                            }

                            let err_info = format!(
                                "值不合法\n字段名：{}, 字段值：{:?}, 第{}行\n字段要求：{}",
                                field.get_field_name(),
                                value.get(i),
                                l,
                                field.get_condition()
                            )
                            .replace("\"", "\\\"")
                            .replace("'", "\\\'")
                            .replace("Some", "");
                            return Err(Box::new(Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "**导表校验失败**\n**项目：{} 文件名：{}**\n{}",
                                    get_project_name(),
                                    name,
                                    err_info
                                ),
                            )));
                        }
                    }
                }
                i += 1;
            }
        }

        Ok(())
    }

    pub fn has_field(&self, file: &String, field: &String, value: &String) -> bool {
        if let Some(xlsx) = self.all.get(file) {
            let mut i = 0usize;
            let field_num = xlsx.field_num();
            let trimd = value.trim();
            while i < field_num {
                if let Some(f) = xlsx.fields.get(i) {
                    if f.get_field_name().eq(field) {
                        for val in &xlsx.values {
                            if let Some(v) = val.get(i) {
                                if v.trim().eq(trimd) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                i += 1;
            }
        }

        false
    }
}

#[cfg(test)]
mod hashmaptests {
    use std::collections::HashMap;

    #[test]
    fn test_string_key() {
        let mut h: HashMap<String, String> = HashMap::new();
        h.insert("test".to_string(), String::from("waht"));
        if let Some(_) = h.get(&String::from("test")) {
            assert!(true);
        } else {
            assert!(false);
        }

        if let Some(_) = h.get(&String::from("xxxx")) {
            assert!(false);
        } else {
            assert!(true);
        }
    }
}
