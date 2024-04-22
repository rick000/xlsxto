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
use crate::{get_project_name, ALLXLSX, XLSX};
use hlua::Lua;
use std::fs;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};

pub struct LuaGenerator<'a> {
    xlsx: &'a XLSX,
}

impl<'a> LuaGenerator<'a> {
    pub fn new(xlsx: &'a XLSX) -> Self {
        LuaGenerator { xlsx }
    }

    fn transfer_lua_keyword(&self, word: &'a String) -> &'a str {
        match word.as_str() {
            "function" => "['function']",
            "end" => "['end']",
            "do" => "['do']",
            "for" => "['for']",
            "if" => "['if']",
            "else" => "['else']",
            "elseif" => "['elseif']",
            "then" => "['then']",
            "while" => "['while']",
            _ => word.as_str(),
        }
    }

    fn type_default_value(&self, t: &String, v: &'a String) -> String {
        let trimd_str = v.trim();
        if !trimd_str.is_empty() {
            return v.clone();
        }

        match t.as_str() {
            "int" | "float" => "0".to_string(),
            "string" => "''".to_string(),

            _ => "nil".to_string(),
        }
    }

    fn get_space_str(&self, space_num: u32) -> String {
        let mut space = String::from("");
        let mut num = space_num;
        while num > 0 {
            space += " ";
            num -= 1;
        }

        space
    }

    fn normalize_key_value(&self, t: &String, v: &String) -> String {
        if t.contains("string") {
            return format!(" [[{}]] ", v);
        }

        return v.clone();
    }

    fn normalize_nonkey_value(
        &self,
        t: &String,
        v: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let type_name = t.as_str();
        if type_name.contains("[]") {
            let value = v.replace("[", "{").replace("]", "}");
            return Ok(self.type_default_value(t, &value));
        }

        if type_name.contains("string") || type_name.contains("localize") {
            let mut double_quote = false;
            let mut single_quote = false;
            let mut right_brace = false;
            let mut has_new_line = false;
            if v.contains("'") {
                single_quote = true;
            }
            if v.contains("\"") {
                double_quote = true;
            }
            if v.contains("]]") || v.ends_with("]") {
                right_brace = true;
            }
            if v.contains("\n") || v.contains("\r") {
                has_new_line = true;
            }

            if !double_quote && !has_new_line {
                return Ok(format!("\"{}\"", v));
            }

            if !single_quote && !has_new_line {
                return Ok(format!("'{}'", v));
            }

            if !right_brace {
                return Ok(format!("[[{}]]", v));
            }

            panic!("string is invalid!{}", v)
        }

        return Ok(self.type_default_value(t, &v));
    }

    pub fn check_lua_file_valid(
        &mut self,
        luastr: &String,
        fname: &String,
        key: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let p = std::panic::catch_unwind(move || {
            let mut lua = Lua::new();
            lua.execute::<()>(luastr)
        })
        .is_err();

        if p {
            let err_info = luastr
                .replace("\"", "\\\"")
                .replace("'", "\\\'")
                .replace("local _ =", "");
            return Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "**导表校验失败**\n**项目：{} 文件名：{}**\n  键值：{} 错误行：{}",
                    get_project_name(),
                    fname,
                    key,
                    err_info
                ),
            )));
        }
        Ok(())
    }

    pub fn generate(
        &mut self,
        fname: &str,
        out_path: &String,
        _allxlsx: &ALLXLSX,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lua_file_name = format!(
            "{}/t_{}.lua",
            out_path,
            fname[..fname.len() - 5].to_lowercase()
        );
        let lua_table_name = format!("t_{}", fname[..fname.len() - 5].to_lowercase());
        let file_name = format!("{}.lua", &lua_table_name);
        let mut file_content = format!(
            "-----------------------------------------------------------------------
-- file: {}
-- desc: this file is generated by tools, do NOT edit this file!
-----------------------------------------------------------------------

{} = 
",
            &file_name, &lua_table_name
        );

        file_content += "{\n";
        let xlsfilename = fname.to_string();

        for row_values in self.xlsx.value_list() {
            let mut line_str = String::from("");
            let mut key_num = 0;
            let mut valid_line = true;
            let mut short_line = String::from("");
            let mut line_key: String = String::from("");
            for field in self.xlsx.fields_list() {
                let value = &row_values[field.as_index() as usize];
                if field.is_key_field() {
                    if value.is_empty() {
                        valid_line = false;
                        break;
                    }

                    key_num += 1;
                    let key = self.normalize_key_value(field.get_field_type(), value);
                    if !field.need_full_load() && !field.is_valid(&key) {
                        let err_info = format!(
                            "键不合法\n字段名：{}, 字段值：{}\n字段要求：{}",
                            field.get_field_name(),
                            key,
                            field.get_condition()
                        )
                        .replace("\"", "\\\"")
                        .replace("'", "\\\'");
                        return Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "**导表校验失败**\n**项目：{} 文件名：{}**\n{}",
                                get_project_name(),
                                fname,
                                err_info
                            ),
                        )));
                    }
                    line_key = format!("{}", &key);
                    let mut key_str = format!("[{}] = ", &key);
                    if key_num == 1 {
                        key_str = self.get_space_str(key_num * 2);
                        key_str += &format!("[{}] = ", &key);
                    }
                    key_str += "{";

                    if short_line.len() + key_str.len() >= 100 {
                        line_str += &short_line;
                        line_str += "\n";
                        short_line = self.get_space_str(key_num * 2 + 2);
                        short_line += &key_str;
                    } else {
                        short_line += &key_str;
                        line_str += &short_line;
                        line_str += "\n";
                        short_line = self.get_space_str(key_num * 2 + 2);
                    }
                } else {
                    let key_str = format!("{} = ", self.transfer_lua_keyword(field.get_field_name()));
                    let value_str =
                        self.normalize_nonkey_value(field.get_field_type(), value)?;
                    if !field.need_full_load() && !field.is_valid(&value_str) {
                        let err_info = format!(
                            "字段值不合要求\n字段名:{}, 字段值:{}\n字段要求: {}",
                            field.get_field_name(),
                            value_str,
                            field.get_condition()
                        )
                        .replace("\"", "\\\"")
                        .replace("'", "\\\'");
                        return Err(Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "**导表校验失败**\n**项目：{} 文件名：{}**\n{}",
                                get_project_name(),
                                fname,
                                err_info
                            ),
                        )));
                    }
                    if short_line.len() + key_str.len() + value_str.len() >= 100 {
                        line_str += &short_line;
                        line_str += "\n";
                        short_line = self.get_space_str(key_num * 2 + 2);
                        short_line += &key_str;
                        short_line += &value_str;
                        short_line += ",";
                    } else {
                        short_line += &key_str;
                        short_line += &value_str;
                        short_line += ",";
                    }
                }
            }

            if valid_line {
                line_str += &short_line;
                while key_num > 0 {
                    line_str += "\n";
                    line_str += &self.get_space_str(key_num * 2);
                    line_str += "}";
                    key_num -= 1;
                }

                line_str += ",\n";

                let mut check_line: String = String::from("local _ = {");
                check_line += &line_str;
                check_line += "}";
                let result = self.check_lua_file_valid(&check_line, &xlsfilename, &line_key);
                if let Err(e) = result {
                    log::error!("invalid config xls={} line = {}", fname, &line_str);
                    return Err(e);
                }
                file_content += &line_str;
            }
        }

        file_content += "}";
        let mut f = fs::File::create(lua_file_name)?;
        f.write(file_content.as_bytes())?;
        Ok(())
    }
}
