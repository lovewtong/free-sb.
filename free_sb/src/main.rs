use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;


fn main() {
    
    // 文件路径
    let folder_path = "";

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file(){
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
fn process_txt_file(path: &Path){
    println!("the txt is: {}", path);

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

fn process_word_file(path: &Path){
    println!("the world is:{}", path);
}

fn process_excel_file(path: &Path){
    println!("the excel is:{}", path);
}
