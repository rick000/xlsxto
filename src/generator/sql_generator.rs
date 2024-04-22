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
use std::fs;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};

use super::Generator;

pub struct SQLGenerator<'a> {
    xlsx: &'a XLSX,
}

impl<'a> SQLGenerator<'a> {
    fn type_default_value(&self, t: &String, v: &'a String) -> String {
        let trimd_str = v.trim();
        if !trimd_str.is_empty() {
            return v.clone();
        }

        match t.as_str() {
            "int" | "float" => "0".to_string(),
            "string" => "''".to_string(),

            _ => "None".to_string(),
        }
    }

    fn normalize_key_value(&self, t: &String, v: &String) -> String {
        if t.contains("string") {
            return format!(" '''{}''' ", v);
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
            return Ok(format!("'{}'", v));
        }

        if type_name.contains("string") {
            let mut double_quote = false;
            let mut single_quote = false;
            if v.contains("'") {
                single_quote = true;
            }
            if v.contains("\"") {
                double_quote = true;
            }

            if !double_quote  {
                return Ok(format!("\"{}\"", v));
            }

            if !single_quote {
                return Ok(format!("'{}'", v));
            }

            return Ok(format!("'{}'", v));
        }

        return Ok(self.type_default_value(t, &v));
    }

    fn xlsxtype_to_mysqltype(&self, type_name: &String) -> String {
        match type_name.as_str() {
            "int" => "INT(11)".to_string(),
            "long" => "BIGINT(20)".to_string(),
            "float" => "DOUBLE".to_string(),
            _ => "text".to_string(),
        }
    }

    fn get_create_table_sql(
        &self,
        table_name: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut create_sql = format!("DROP TABLE IF EXISTS `{}`;\n", table_name);
        create_sql += &format!("CREATE TABLE `{}` (\n", table_name);
        let key_num = self.xlsx.key_num();
        let mut prikey_str = "".to_string();
        if key_num > 0 {
            prikey_str += "  PRIMARY KEY(";
        }
        let mut prifx = "  ".to_string();
        let mut key_prefix = "".to_string();
        for field in self.xlsx.fields_list() {
            let field_sql = format!(
                "{}`{}` {}{}", prifx,
                field.get_field_name(),
                self.xlsxtype_to_mysqltype(field.get_field_type()),
                if field.is_key_field() { " NOT NULL" } else { "" }
            );
            create_sql += &field_sql;
            prifx = ",\n  ".to_string();
            if field.is_key_field() {
                prikey_str += &format!("{}`{}`", key_prefix, field.get_field_name());
                key_prefix = ",".to_string();
            }
        }
        if key_num > 0 {
            create_sql += ",\n";
            prikey_str += ")";
            create_sql += &prikey_str;
        }

        
        create_sql += "\n);\n";
        Ok(create_sql)
    }
}

impl<'a> Generator<'a> for SQLGenerator<'a> {
    fn new(xlsx: &'a XLSX) -> Self {
        SQLGenerator { xlsx }
    }

    fn generate(
        &mut self,
        fname: &str,
        out_path: &String,
        _allxlsx: &ALLXLSX,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sql_file_name = format!(
            "{}/t_{}.sql",
            out_path,
            fname[..fname.len() - 5].to_lowercase()
        );

        let table_name = format!("t_{}", fname[..fname.len() - 5].to_lowercase());
        let mut file_content = self.get_create_table_sql(&table_name)?;
        

        for row_values in self.xlsx.value_list() {
            let mut key_part = format!("INSERT INTO `{}`(", table_name);
            let mut value_part = " VALUES(".to_string();
            let mut prefix = "".to_string();
            for field in self.xlsx.fields_list() {
                let value = &row_values[field.as_index() as usize];
                if field.is_key_field() && value.is_empty() {
                    break;
                }

                key_part += &format!("{}`{}`", prefix, field.get_field_name());
                value_part += &format!("{}{}", prefix, self.normalize_nonkey_value(field.get_field_type(), value)?);
                
                prefix = ",".to_string();

                if field.is_key_field() {
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
                } else {
                    let value_str = self.normalize_nonkey_value(field.get_field_type(), value)?;
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
                }
            }

            file_content += &format!("{}) {});\n", key_part, value_part);
        }

        let mut f = fs::File::create(sql_file_name)?;
        f.write(file_content.as_bytes())?;
        Ok(())
    }
}
