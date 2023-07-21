

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    InvalidPartitionSpecification,
    InvalidDevice,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::ParseIntError(error)
    }
}


pub type Result<T> = std::result::Result<T, Error>;


pub fn parse_size(size: &str) -> Result<u64> {
    let size = size.trim();

    let size = if size.ends_with('K') {
        size.trim_end_matches('K').parse::<u64>()? * 1024
    } else if size.ends_with('M') {
        size.trim_end_matches('M').parse::<u64>()? * 1024 * 1024
    } else if size.ends_with('G') {
        size.trim_end_matches('G').parse::<u64>()? * 1024 * 1024 * 1024
    } else {
        size.parse::<u64>()?
    };

    Ok(size)
}
