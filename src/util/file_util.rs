use failure::Error;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct FileUtil;

impl FileUtil {
    pub fn write_temp_file(file_name: &str, content: &str) -> Result<PathBuf, Error> {
        let temp_directory = env::temp_dir();
        let temp_file = temp_directory.join(file_name);

        let mut file = File::create(temp_file.clone())?;
        file.write_all(content.as_bytes())?;

        Ok(temp_file)
    }

    pub fn write_text_file(file_path: &PathBuf, content: &str) -> Result<(), Error> {
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
