use docx_rs::{read_docx, Docx, Paragraph, Run, Text};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // 文件路径
    let folder_path = r"C:\Users\WU\Desktop\mytest";

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("txt") => process_txt_file(path),
                Some("word") => process_word_file(path),
                Some("excel") => process_excel_file(path),
                _ => println!("error! it not able file: {:?}", path),
            }
        }
    }
}

// 处理txt文件
fn process_txt_file(path: &Path) {
    println!("the txt is: {}", path.display());

    // 读取文件内容
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open the file {:?}:{}", path, err);
            return;
        }
    };

    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        eprintln!("Failed to read file {:?}:{}", path, err);
        return;
    }

    // 替换关键词
    let new_content = content.replace("20240716", "20240717");

    // 写回文件
    let mut file = match fs::File::create(path) {
        Ok(file) => file,
        Err(err) => {
            eprint!("Failed to create the file {:?}:{}", path, err);
            return;
        }
    };

    if let Err(err) = file.write_all(new_content.as_bytes()) {
        eprintln!("failed to write the file {:?}:{}", path, err);
    }
}

// 处理DOCX文件
fn process_word_file(path: &Path) {
    println!("the world is:{}", path.display());

     // 打开 DOCX 文件
     let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file {}: {:?}", path.display(), err);
            return;
        }
    };

    // 读取文件内容到字节向量中
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    if let Err(err) = reader.read_to_end(&mut buffer) {
        eprintln!("Failed to read DOCX file {}: {:?}", path.display(), err);
        return;
    }

    // 读取 DOCX 文件内容
    let mut docx = match read_docx(&buffer[..]) {
        Ok(docx) => docx,
        Err(err) => {
            eprintln!("Failed to read DOCX file {}: {:?}", path.display(), err);
            return;
        }
    };

    // 替换关键词
    for para in docx.document.body_mut().paragraphs_mut() {
        for run in para.runs_mut() {
            if let Some(text) = run.text_mut() {
                if text == "20240716" {
                    *text = "20240717".to_string();
                }
            }
        }
    }

    // 写回文件
    let mut out_file = match File::create(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to create file {}: {:?}", path.display(), err);
            return;
        }
    };

    if let Err(err) = docx.write(&mut out_file) {
        eprintln!("Failed to write the DOCX {}: {:?}", path.display(), err);
    }
}

fn process_excel_file(path: &Path) {
    println!("the excel is:{}", path.display());
}
