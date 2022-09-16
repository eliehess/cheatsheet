use std::env::{args, current_exe};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use pulldown_cmark::{Parser, Options, html};

macro_rules! exit {
    ($($x:expr),*) => ({
        eprintln!($($x),*);
        std::process::exit(0);
    });
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        exit!("Usage: cheatsheet <name>");
    }

    let dir = match current_exe() {
        Err(_) =>  exit!("Unable to get current folder"),
        Ok(mut exe) => {
            exe.pop();
            exe
        }
    };

    let files: Vec<PathBuf> = match get_matching_files(&dir, &args[1]) {
        Err(e) => exit!("{}", e),
        Ok(files) => files
    };

    let mut file: PathBuf = match files.len() {
        0 => exit!("No matching files found"),
        1 => files[0].to_path_buf(),
        _ => {
            match check_markdown_condition(&files) {
                Some(file) => file.to_path_buf(),
                None => exit!("{}", create_multi_file_error(&files))
            }
        }
    };

    if let Some(ext) = file.extension().and_then(OsStr::to_str) {
        if ext == "md" {
            file = match convert_markdown_to_html(&file) {
                Err(e) => exit!("{}", e),
                Ok(f) => f
            };
        }
    }

    let filename = match file.as_path().file_name().and_then(OsStr::to_str) {
        None => exit!("Unable to get filename"),
        Some(f) => f
    };

    match opener::open(&file) {
        Err(e) => exit!("{}", e),
        Ok(()) => println!("Opened {}", filename)
    };
}

fn get_matching_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    if !dir.is_dir() {
        return Err(String::from("You must provide a path to a directory, not to a file"));
    }

    let read_dir = match fs::read_dir(dir) {
        Err(e) => return Err(e.to_string()),
        Ok(read_dir) => read_dir
    };

    Ok(read_dir
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap().path())
        .filter(|pathbuf| pathbuf.is_file() && pathbuf_contains_pattern_ignore_case(pathbuf, pattern))
        .collect()
    )
}

fn pathbuf_contains_pattern_ignore_case(pathbuf: &PathBuf, pattern: &str) -> bool {
    match pathbuf.file_stem().and_then(OsStr::to_str) {
        None => false,
        Some(string) => string.to_lowercase().contains(&(pattern.to_lowercase()))
    }
}

fn check_markdown_condition(files: &Vec<PathBuf>) -> Option<&PathBuf> {
    let mut markdown: Option<&PathBuf> = None;
    for file in files {
        match file.extension().and_then(OsStr::to_str) {
            None => return None,
            Some(ext) => match ext {
                "md" => match markdown {
                    Some(_) => return None,
                    None => markdown = Some(file)
                },
                "html" => (),
                _ => return None
            }
        }
    }
    markdown
}

fn convert_markdown_to_html(markdown: &PathBuf) -> io::Result<PathBuf> {
    let markdown_input = fs::read_to_string(markdown)?;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let new_file_path = markdown.with_extension("html");
    let mut output_file = File::create(&new_file_path)?;
    output_file.write_all(html_output.as_bytes())?;

    Ok(new_file_path)
}

fn create_multi_file_error(files: &Vec<PathBuf>) -> String {
    let mut err = String::from("Multiple matching files found, please refine your search or rename them:"); 
    for file in files {
        if let Some(f) = file.as_path().file_name().and_then(OsStr::to_str) {
            err = format!("{}\n\t{}", err, f)
        }
    }
    err
}
