use docx_rs::{read_docx, DocumentChild, Docx, Paragraph, Run, Text, ParagraphChild, RunChild};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use walkdir::WalkDir;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};


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

    // 遍历文档的 children
    for child in &mut docx.document.children {
        if let DocumentChild::Paragraph(ref mut paragraph) = child {
            // 遍历段落的 children
            for run in &mut paragraph.children {
                if let ParagraphChild::Run(ref mut run) = run {
                    // 遍历 Run 的 children
                    for run_child in &mut run.children {
                        if let RunChild::Text(ref mut text) = run_child {
                            if text.text == "20240716" {
                                text.text = "20240717".to_string();
                            }
                        }
                    }
                }
            }
        }
    }
   // 使用 XMLBuilder 写入内容
   let xml_builder = XMLBuilder::new();

   // 这里你需要实现将 Docx 转换为 XML 的逻辑
   let xml_content = build_xml_from_docx(&docx, xml_builder);

   // 写回文件
   let out_file = match File::create(path) {
       Ok(file) => file,
       Err(err) => {
           eprintln!("Failed to create file {}: {:?}", path.display(), err);
           return;
       }
   };
   
   let mut writer = BufWriter::new(out_file);
   if let Err(err) = writer.write_all(&xml_content) {
       eprintln!("Failed to write DOCX file {}: {:?}", path.display(), err);
   }
}

fn build_xml_from_docx(docx: &Docx, mut builder: XMLBuilder) -> Vec<u8> {
   // Implement XML building from Docx content here
   // This function should convert the Docx content into XML and return it as Vec<u8>
   // This is a placeholder function
   builder.build()
}

fn process_excel_file(path: &Path) {
    println!("the excel is:{}", path.display());
}
