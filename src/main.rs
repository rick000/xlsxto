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

mod checker;
mod generator;
mod xlsx;

use clap::Parser;
use curl::easy::{Easy, List};
use std::error::Error;
use std::fs;
use std::io::Read;
use xlsx::{ALLXLSX, XLSX};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input path to specific
    #[arg(short, long, default_value_t = String::from("."))]
    input: String,

    /// output path to specific
    #[arg(short, long, default_value_t = String::from("."))]
    output: String,

    #[arg(short, long, default_value_t = String::from("lua"))]
    target: String,

    #[arg(short, long, default_value_t = String::from(""))]
    notify_url: String,

    #[arg(short, long, default_value_t = String::from(""))]
    project: String,
}

fn notify_error_info(url: &String, e: &Box<dyn Error>) -> Result<(), Box<dyn Error>> {
    if url.is_empty() || !url.starts_with("http") {
        log::error!("invalid notifiy url {}! error {}", url, e);
        return Ok(());
    }

    let mut easy = Easy::new();
    easy.url(url)?;

    let mut list = List::new();
    list.append("Content-Type: application/json")?;
    easy.http_headers(list)?;
    easy.post(true)?;

    let mut post_info = String::from("{\"msgtype\":\"markdown\", \"markdown\":{\"content\":\"");
    post_info += &format!("{}", e);
    post_info += "\"";
    let mut data = post_info.as_bytes();

    easy.post_field_size(data.len() as u64)?;
    let mut transfer = easy.transfer();
    transfer.read_function(|buf| Ok(data.read(buf).unwrap_or(0)))?;
    transfer.perform()?;
    Ok(())
}

pub fn get_project_name() -> String {
    let args = Args::parse();
    return args.project;
}

const PROG_SIG: &str = r"
       .__                    __                          .__        __   .__                   
___  __|  |   _________  ____/  |_  ____           _______|__| ____ |  | _|  |__ _____    ____  
\  \/  /  |  /  ___/\  \/  /\   __\/  _ \   ______ \_  __ \  |/ ___\|  |/ /  |  \\__  \  /    \ 
 >    <|  |__\___ \  >    <  |  | (  <_> ) /_____/  |  | \/  \  \___|    <|   Y  \/ __ \|   |  \
/__/\_ \____/____  >/__/\_ \ |__|  \____/           |__|  |__|\___  >__|_ \___|  (____  /___|  /
      \/         \/       \/                                      \/     \/    \/     \/     \/ 
";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();
    println!(
        "{} xlsx path={}, output path={}",
        PROG_SIG, args.input, args.output
    );

    let mut all = ALLXLSX::new();

    for entry in fs::read_dir(&args.input).expect("directory should be exists!") {
        let path = entry?.path();
        let metadata = fs::metadata(&path)?;
        if metadata.is_file() {
            let fname = path.file_name().unwrap().to_str().unwrap();
            if fname.ends_with(".xls") || fname.ends_with(".xlsx") {
                let mut xlsx = XLSX::new();
                xlsx.parse_from_file(&args.input, &fname, false, &args.target)?;
                all.add(fname, xlsx);
            }
        }
    }

    if args.target == "lua" {
        let result = all.gen_lua(&args.output);
        if let Err(e) = result {
            let notify_error = notify_error_info(&args.notify_url, &e);
            if let Err(s) = notify_error {
                log::error!("notify error occurred! {}", s);
            }
            return Err(e);
        }
    }
    if args.target == "json" {
        let result = all.gen_json(&args.output);
        if let Err(e) = result {
            let notify_error = notify_error_info(&args.notify_url, &e);
            if let Err(s) = notify_error {
                log::error!("notify error occurred! {}", s);
            }
            return Err(e);
        }
    }

    let result = all.check_xlsx_valid();
    if let Err(e) = result {
        let notify_error = notify_error_info(&args.notify_url, &e);
        if let Err(s) = notify_error {
            log::error!("notify error occurred! {}", s);
        }
        return Err(e);
    }
    Ok(())
}
