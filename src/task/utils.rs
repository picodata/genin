use std::{fs::File, io, path::PathBuf};

use log::warn;

#[allow(unused)]
pub fn uncolorize<S: ToString>(s: S) -> String {
    String::from_utf8(strip_ansi_escapes::strip(s.to_string()).unwrap()).unwrap()
}

pub fn create_file_or_copy(path: PathBuf, force: bool) -> Result<File, io::Error> {
    if !path.exists() || force {
        File::create(path)
    } else if path.is_file() {
        let file_ext = path
            .extension()
            .expect("Failed to get file extension")
            .to_str()
            .expect("Failed to cast into str");
        let file_name = path
            .file_stem()
            .expect("Failed to get filename")
            .to_str()
            .expect("Failed to cast into str");

        warn!(
            "the target file {} already exists so the new file will be \
                                saved with name {file_name}.copy.{file_ext}",
            path.display(),
        );

        File::create(format!("{file_name}.copy.{file_ext}"))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "output is not file or not valid path",
        ))
    }
}
