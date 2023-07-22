use std::fs::OpenOptions;

use crate::util::{Result, Size};


pub fn create_image(path: &str, size: &str) -> Result<()> {
    let size = Size::parse(size)?;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)?;
    file.set_len(size.to_bytes())?;
    Ok(())
}
