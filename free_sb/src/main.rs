use std::fs::{self, File};
use std::io::{Read, Write, stdin, stdout};
use std::path::Path;
use walkdir::WalkDir;
use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use quick_xml::events::{Event, BytesText};
use quick_xml::Reader;
use quick_xml::Writer;

fn main() {
    // Prompt user for inputs
    let mut old_text = String::new();
    let mut new_text = String::new();

    print!("Enter the text to replace: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut old_text).unwrap();
    let old_text = old_text.trim().to_string();

    print!("Enter the replacement text: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut new_text).unwrap();
    let new_text = new_text.trim().to_string();

    // Get the folder path
    let mut folder_path = String::new();
    print!("Enter the folder path to process: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut folder_path).unwrap();
    let folder_path = folder_path.trim();

    // Iterate over files in the folder
    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            match path.extension().and_then(|s| s.to_str()) {
                Some("txt") => process_txt_file(path, &old_text, &new_text),
                Some("docx") => process_word_file(path, &old_text, &new_text),
                Some("xlsx") => process_excel_file(path, &old_text, &new_text),
                _ => println!("Unsupported file: {:?}", path),
            }
        }
    }
}

fn process_txt_file(path: &Path, old_text: &str, new_text: &str) {
    println!("Processing txt file: {}", path.display());

    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open the file {:?}: {}", path, err);
            return;
        }
    };

    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        eprintln!("Failed to read file {:?}: {}", path, err);
        return;
    }

    let new_content = content.replace(old_text, new_text);

    if content != new_content {
        let mut file = match fs::File::create(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to create the file {:?}: {}", path, err);
                return;
            }
        };

        if let Err(err) = file.write_all(new_content.as_bytes()) {
            eprintln!("Failed to write the file {:?}: {}", path, err);
        }
    }
}

fn process_word_file(path: &Path, old_text: &str, new_text: &str) {
    println!("Processing docx file: {}", path.display());

    let temp_path = path.with_extension("tmp");

    // Ensure all file handles are dropped before renaming
    {
        // Open the docx file as a zip archive
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open file {}: {:?}", path.display(), err);
                return;
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => archive,
            Err(err) => {
                eprintln!("Failed to read ZIP archive {}: {:?}", path.display(), err);
                return;
            }
        };

        // Read and modify document.xml
        let mut content = String::new();
        {
            let mut file = match archive.by_name("word/document.xml") {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "Failed to find document.xml in {}: {:?}",
                        path.display(),
                        err
                    );
                    return;
                }
            };

            if let Err(err) = file.read_to_string(&mut content) {
                eprintln!(
                    "Failed to read document.xml in {}: {:?}",
                    path.display(),
                    err
                );
                return;
            }
        }

        // Parse and modify XML content
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);
        let mut writer = Writer::new(Vec::new());

        let mut buf = Vec::new();
        let mut text_accumulator = String::new();
        let mut inside_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(event) => match event {
                    Event::Start(ref e) => {
                        if inside_text {
                            let replaced_text = text_accumulator.replace(old_text, new_text);
                            writer
                                .write_event(Event::Text(BytesText::new(&replaced_text)))
                                .unwrap();
                            text_accumulator.clear();
                            inside_text = false;
                        }
                        writer.write_event(Event::Start(e.clone())).unwrap();
                    }
                    Event::End(ref e) => {
                        if inside_text {
                            let replaced_text = text_accumulator.replace(old_text, new_text);
                            writer
                                .write_event(Event::Text(BytesText::new(&replaced_text)))
                                .unwrap();
                            text_accumulator.clear();
                            inside_text = false;
                        }
                        writer.write_event(Event::End(e.clone())).unwrap();
                    }
                    Event::Text(e) => {
                        let text = e.unescape().unwrap();
                        text_accumulator.push_str(&text);
                        inside_text = true;
                    }
                    Event::Eof => break,
                    _ => {
                        writer.write_event(event).unwrap();
                    }
                },
                Err(e) => {
                    eprintln!("Error reading XML: {:?}", e);
                    return;
                }
            }
            buf.clear();
        }

        // Get modified XML content
        let result = writer.into_inner();

        // Create a new ZIP file with the modified content
        {
            let output_file = match File::create(&temp_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "Failed to create temporary file {}: {:?}",
                        temp_path.display(),
                        err
                    );
                    return;
                }
            };

            let mut zip = ZipWriter::new(output_file);

            // Copy and replace files in the zip archive
            for i in 0..archive.len() {
                let mut file = match archive.by_index(i) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!(
                            "Failed to read file in archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }
                };

                let options = FileOptions::default()
                    .compression_method(CompressionMethod::Deflated)
                    .unix_permissions(file.unix_mode().unwrap_or(0o755));

                if file.name() == "word/document.xml" {
                    if let Err(err) = zip.start_file(file.name(), options) {
                        eprintln!(
                            "Failed to start file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    if let Err(err) = zip.write_all(&result) {
                        eprintln!(
                            "Failed to write document.xml in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }
                } else {
                    if let Err(err) = zip.start_file(file.name(), options) {
                        eprintln!(
                            "Failed to start file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    let mut buffer = Vec::new();
                    if let Err(err) = file.read_to_end(&mut buffer) {
                        eprintln!(
                            "Failed to read file in archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    if let Err(err) = zip.write_all(&buffer) {
                        eprintln!(
                            "Failed to write file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }
                }
            }

            if let Err(err) = zip.finish() {
                eprintln!("Failed to finish zip archive {}: {:?}", path.display(), err);
                return;
            }
        }
    } // Ensure all file handles are closed here

    // Now, replace the original file with the modified file
    if let Err(err) = fs::rename(&temp_path, path) {
        eprintln!(
            "Failed to replace original file with modified file: {:?}",
            err
        );
    }
}

fn process_excel_file(path: &Path, old_text: &str, new_text: &str) {
    println!("Processing xlsx file: {}", path.display());

    let temp_path = path.with_extension("tmp");

    // Ensure all file handles are dropped before renaming
    {
        // Open the xlsx file as a zip archive
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to open file {}: {:?}", path.display(), err);
                return;
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => archive,
            Err(err) => {
                eprintln!("Failed to read ZIP archive {}: {:?}", path.display(), err);
                return;
            }
        };

        // Collect the names of the sheet XML files
        let mut sheet_files = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            let name = file.name().to_string();
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                sheet_files.push(name);
            }
        }

        // For each sheet file, read and modify its content
        let mut modified_sheets = std::collections::HashMap::new();
        for sheet_name in &sheet_files {
            let mut content = String::new();
            {
                let mut file = match archive.by_name(&sheet_name) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!(
                            "Failed to find {} in {}: {:?}",
                            sheet_name,
                            path.display(),
                            err
                        );
                        return;
                    }
                };

                if let Err(err) = file.read_to_string(&mut content) {
                    eprintln!(
                        "Failed to read {} in {}: {:?}",
                        sheet_name,
                        path.display(),
                        err
                    );
                    return;
                }
            }

            // Parse and modify XML content
            let mut reader = Reader::from_str(&content);
            reader.config_mut().trim_text(true);
            let mut writer = Writer::new(Vec::new());

            let mut buf = Vec::new();
            let mut text_accumulator = String::new();
            let mut inside_text = false;

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(event) => match event {
                        Event::Start(ref e) => {
                            if inside_text {
                                let replaced_text =
                                    text_accumulator.replace(old_text, new_text);
                                writer
                                    .write_event(Event::Text(BytesText::new(&replaced_text)))
                                    .unwrap();
                                text_accumulator.clear();
                                inside_text = false;
                            }
                            writer.write_event(Event::Start(e.clone())).unwrap();
                        }
                        Event::End(ref e) => {
                            if inside_text {
                                let replaced_text =
                                    text_accumulator.replace(old_text, new_text);
                                writer
                                    .write_event(Event::Text(BytesText::new(&replaced_text)))
                                    .unwrap();
                                text_accumulator.clear();
                                inside_text = false;
                            }
                            writer.write_event(Event::End(e.clone())).unwrap();
                        }
                        Event::Text(e) => {
                            let text = e.unescape().unwrap();
                            text_accumulator.push_str(&text);
                            inside_text = true;
                        }
                        Event::Eof => break,
                        _ => {
                            writer.write_event(event).unwrap();
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading XML: {:?}", e);
                        return;
                    }
                }
                buf.clear();
            }

            // Get the modified XML content
            let result = writer.into_inner();

            // Save the modified content
            modified_sheets.insert(sheet_name.clone(), result);
        }

        // Create a new ZIP file with the modified content
        {
            let output_file = match File::create(&temp_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "Failed to create temporary file {}: {:?}",
                        temp_path.display(),
                        err
                    );
                    return;
                }
            };

            let mut zip = ZipWriter::new(output_file);

            for i in 0..archive.len() {
                let mut file = match archive.by_index(i) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!(
                            "Failed to read file in archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }
                };

                let options = FileOptions::default()
                    .compression_method(CompressionMethod::Deflated)
                    .unix_permissions(file.unix_mode().unwrap_or(0o755));

                if let Some(result) = modified_sheets.get(file.name()) {
                    if let Err(err) = zip.start_file(file.name(), options) {
                        eprintln!(
                            "Failed to start file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    if let Err(err) = zip.write_all(result) {
                        eprintln!(
                            "Failed to write {} in zip archive {}: {:?}",
                            file.name(),
                            path.display(),
                            err
                        );
                        return;
                    }
                } else {
                    if let Err(err) = zip.start_file(file.name(), options) {
                        eprintln!(
                            "Failed to start file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    let mut buffer = Vec::new();
                    if let Err(err) = file.read_to_end(&mut buffer) {
                        eprintln!(
                            "Failed to read file in archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }

                    if let Err(err) = zip.write_all(&buffer) {
                        eprintln!(
                            "Failed to write file in zip archive {}: {:?}",
                            path.display(),
                            err
                        );
                        return;
                    }
                }
            }

            if let Err(err) = zip.finish() {
                eprintln!("Failed to finish zip archive {}: {:?}", path.display(), err);
                return;
            }
        }
    } // Ensure all file handles are closed here

    // Now, replace the original file with the modified file
    if let Err(err) = fs::rename(&temp_path, path) {
        eprintln!(
            "Failed to replace original file with modified file: {:?}",
            err
        );
    }
}
