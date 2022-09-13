use std::env::{args, current_exe};
use std::fs;
use std::path::{Path, PathBuf};
use opener::open;

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

    let files = match get_matching_files(&dir, &args[1]) {
        Err(e) => exit!("{}", e),
        Ok(files) => files
    };

    let file = match files.len() {
        0 => exit!("No matching files found"),
        1 => &files[0],
        _ => {
            let mut err = String::from("Multiple matching files found, please refine your search or rename them:"); 
            for file in files { err = format!("{}\n\t{}", err, file.display()) }
            exit!("{}", err);
        }
    };

    match open(file) {
        Err(e) => exit!("{}", e),
        Ok(()) => println!("Opened {}", file.display())
    }
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
    match pathbuf.file_stem() {
        None => false,
        Some(os_str) => {
            match os_str.to_str() {
                None => false,
                Some(string) => string.to_lowercase().contains(&(pattern.to_lowercase()))
            }
        }
    }
}
