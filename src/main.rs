use std::collections::VecDeque;
use std::env::{args, current_exe};
use std::fs;
use std::path::{Path, PathBuf};
use opener::open;

fn main() {
    let args: VecDeque<String> = args().collect();

    if args.len() != 2 {
        println!("Usage: cheatsheet <name>");
        return;
    }

    let mut dir = current_exe().unwrap();
    dir.pop();
    match get_matching_files(&dir, &args[1]) {
        Ok(files) => {
            match files.len() {
                1 => { open(format!("{}", files[0].display())).unwrap(); },
                0 => { println!("No files found"); },
                _ => { 
                    println!("Multiple matching files found, please refine your search or rename them:"); 
                    for f in files { println!("\t{}", f.display())} 
                }
            }
        },
        Err(e) => { println!("{}", e) }
    };
}

fn get_matching_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    Ok(get_files(dir)?
        .into_iter()
        .filter(|pathbuf| pathbuf_contains_pattern(pathbuf, pattern))
        .collect()
    )
}

fn get_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.is_dir() {
        return Err(String::from("You must provide a path to a directory, not to a file"));
    }

    match fs::read_dir(dir) {
        Err(e) => { return Err(e.to_string()) },
        Ok(dir) => {
            Ok(dir
                .filter(|result| result.is_ok())
                .map(|result| result.unwrap().path())
                .filter(|pathbuf| pathbuf.is_file())
                .collect()
            )
        }
    }
}

fn pathbuf_contains_pattern(pathbuf: &PathBuf, pattern: &str) -> bool {
    match pathbuf.file_stem() {
        None => false,
        Some(os_str) => {
            match os_str.to_str() {
                None => false,
                Some(string) => string.contains(pattern)
            }
        }
    }
}
