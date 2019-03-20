use std::fs;
use std::path::Path;

pub fn load_file(file: &Path) -> Option<String> {
    let contents = fs::read_to_string(file);
    match contents {
        Ok(file_str) => Some(file_str),
        Err(err) => {
            #[cfg(feature = "debug")]
            eprintln!("[ERR] Impossible to read file {} : {}", file.display(), err);

            None
        }
    }
}
