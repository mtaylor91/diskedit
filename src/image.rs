use std::fs::OpenOptions;

use crate::util::{parse_size, Result};


pub fn create_image(path: &str, size: &str) -> Result<()> {
    let size = parse_size(size)?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)?;
    file.set_len(size)?;
    Ok(())
}
