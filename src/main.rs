use pulldown_cmark::{html, Options, Parser};
use std::env::{args, current_exe};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    match do_main() {
        Err(e) => eprintln!("{}", e),
        Ok(s) => println!("{}", s),
    };
}

fn do_main() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        return Ok("Usage: cheatsheet <name>".into());
    }

    let dir = {
        let mut exe = current_exe()?;
        exe.pop();
        exe
    };

    let files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap().path())
        .filter(|pathbuf| {
            pathbuf.is_file() && pathbuf_contains_pattern_ignore_case(pathbuf, &args[1])
        })
        .collect();

    let mut file: PathBuf = match files.len() {
        0 => return Ok("No matching files found".to_owned()),
        1 => files[0].to_path_buf(),
        _ => match check_markdown_condition(&files) {
            Some(file) => file.to_path_buf(),
            None => contains_exact_string(&files, &args[1]).ok_or(multi_file_error(&files))?,
        },
    };

    if let Some(ext) = file.extension().and_then(OsStr::to_str) {
        if ext == "md" {
            file = convert_markdown_to_html(&file)?;
        }
    }

    let filename = file
        .as_path()
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or("Unable to get filename")?;

    opener::open(&file)?;

    Ok(format!("Opened {}", filename))
}

fn pathbuf_contains_pattern_ignore_case(pathbuf: &PathBuf, pattern: &str) -> bool {
    match pathbuf.file_stem().and_then(OsStr::to_str) {
        None => false,
        Some(string) => string.to_lowercase().contains(&(pattern.to_lowercase())),
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
                    None => markdown = Some(file),
                },
                "html" => (),
                _ => return None,
            },
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

fn multi_file_error(files: &Vec<PathBuf>) -> String {
    let mut err =
        String::from("Multiple matching files found, please refine your search or rename them:");
    for file in files {
        if let Some(f) = file.as_path().file_name().and_then(OsStr::to_str) {
            err = format!("{}\n\t{}", err, f)
        }
    }
    err
}

fn contains_exact_string(files: &Vec<PathBuf>, string: &str) -> Option<PathBuf> {
    for file in files {
        if let Some(filename) = file.file_stem().and_then(OsStr::to_str) {
            if filename.eq(string) {
                return Some(file.to_path_buf());
            }
        }
    }
    None
}
