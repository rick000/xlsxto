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
use super::Generator;
use crate::{ALLXLSX, XLSX};
use std::fs;
use std::io::prelude::*;

pub struct JsonGenerator<'a> {
    xlsx: &'a XLSX,
}

fn replace_json_br(s: &str) -> String {
    return s
        .replace("\r", "")
        .replace("\\n", "\n")
        .replace("\n", "\\n");
}

impl<'a> JsonGenerator<'a> {
    fn type_default_value(&self, t: &String, v: &String) -> String {
        let trimd_str = v.trim();
        if !trimd_str.is_empty() {
            return v.clone();
        }

        match t.as_str() {
            "int" | "float" | "long" => "0".to_string(),
            "string" => "\"\"".to_string(),

            _ => "nil".to_string(),
        }
    }

    pub fn normalize_values(
        &self,
        field_type: &String,
        value: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut ret_value = String::from("");
        if field_type.contains("string") {
            if field_type.contains("[]") {
                ret_value += "[";
                let mut prefix = "";
                let s = value.replace("[", "").replace("]", "");
                let content: Vec<&str> = s.split(",").collect();
                for c in content {
                    ret_value += prefix;
                    if c.contains("\"") {
                        ret_value += &replace_json_br(c);
                    } else {
                        ret_value += "\"";
                        ret_value += &replace_json_br(c);
                        ret_value += "\"";
                    }
                    prefix = ",";
                }
                ret_value += "]";
            } else {
                if ret_value.contains("\"") {
                    ret_value = replace_json_br(value.as_str());
                } else {
                    ret_value += "\"";
                    ret_value += &replace_json_br(value.as_str());
                    ret_value += "\"";
                }
            }
        } else if field_type.contains("[]") {
            ret_value += "[";
            let mut prefix = "";
            let s = value.replace("[", "").replace("]", "");
            let content: Vec<&str> = s.split(",").collect();
            for c in content {
                ret_value += prefix;
                ret_value += c;
                prefix = ",";
            }
            ret_value += "]";
        } else {
            ret_value = self.type_default_value(field_type, value);
        }

        Ok(ret_value)
    }
}

impl<'a> Generator<'a> for JsonGenerator<'a> {
    fn new(xlsx: &'a XLSX) -> Self {
        JsonGenerator { xlsx }
    }

    fn generate(
        &mut self,
        fname: &str,
        out_path: &String,
        _allxlsx: &ALLXLSX,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_file_name = format!("{}/{}.json", out_path, fname[..fname.len() - 5].to_string());
        let mut file_content = String::from("[");

        let mut line_prefix = String::from("");
        for row_values in self.xlsx.value_list() {
            let mut line_content = line_prefix + "\n{";

            let mut column_prefix = String::from("");
            for field in self.xlsx.fields_list() {
                let v = self.normalize_values(
                    field.get_field_type(),
                    &row_values[field.as_index() as usize],
                )?;
                line_content += &column_prefix;
                line_content += "\"";
                let name = field.get_field_name().clone();
                line_content += &name;
                line_content += "\":";
                line_content += &v;
                column_prefix = String::from(",");
            }

            line_content += "}";
            line_prefix = String::from(",");
            file_content += &line_content;
        }

        file_content += "\n]";

        let mut f = fs::File::create(json_file_name)?;
        f.write(file_content.as_bytes())?;
        Ok(())
    }
}
