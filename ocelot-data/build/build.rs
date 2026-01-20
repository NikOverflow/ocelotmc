mod registry;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use proc_macro2::TokenStream;

pub const OUT_DIR: &str = "src/generated";

pub fn main() {
    std::fs::create_dir_all(OUT_DIR).unwrap();

    let build_functions: Vec<(fn() -> TokenStream, &str)> = vec![(registry::build, "registry.rs")];

    build_functions.iter().for_each(|(build_fn, file)| {
        let raw_code = build_fn().to_string();
        let final_code = format_code(&raw_code);
        write_generated_file(&final_code, file);
    });
}

pub fn write_generated_file(new_code: &str, out_file: &str) {
    let path = Path::new(OUT_DIR).join(out_file);
    if path.exists()
        && let Ok(existing_code) = std::fs::read_to_string(&path)
        && existing_code == new_code
    {
        return;
    }
    std::fs::write(&path, new_code).unwrap();
}

pub fn format_code(unformatted_code: &str) -> String {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(unformatted_code.as_bytes())
        .unwrap();
    String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
}
