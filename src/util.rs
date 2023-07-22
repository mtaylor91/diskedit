

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    InvalidDevice,
    InvalidOffset,
    InvalidPartitionType,
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


#[derive(Clone, Copy, Debug)]
pub enum Offset {
    Percent(u8),
    Size(Size),
}

impl Offset {
    pub fn parse(offset: &str) -> Result<Self> {
        let offset = offset.trim();

        if offset.ends_with('%') {
            let percent = offset.trim_end_matches('%').parse::<u8>()?;
            if percent > 100 {
                return Err(Error::InvalidOffset);
            }
            Ok(Offset::Percent(percent))
        } else {
            let size = Size::parse(offset)?;
            Ok(Offset::Size(size))
        }
    }

    pub fn to_bytes(&self, total: u64) -> u64 {
        match self {
            Offset::Percent(percent) => total * (*percent as u64) / 100,
            Offset::Size(s) => s.to_bytes(),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Size {
    Bytes(u64),
    Kibibytes(u64),
    Kilobytes(u64),
    Mebibytes(u64),
    Megabytes(u64),
    Gibibytes(u64),
    Gigabytes(u64),
}

impl Size {
    pub fn parse(size: &str) -> Result<Self> {
        let size = size.trim();

        if size.ends_with("kB") {
            let kilobytes = size.trim_end_matches("kB").parse::<u64>()?;
            Ok(Size::Kilobytes(kilobytes))
        } else if size.ends_with("kiB") {
            let kibibytes = size.trim_end_matches("kiB").parse::<u64>()?;
            Ok(Size::Kibibytes(kibibytes))
        } else if size.ends_with("mB") {
            let megabytes = size.trim_end_matches("mB").parse::<u64>()?;
            Ok(Size::Megabytes(megabytes))
        } else if size.ends_with("miB") {
            let mebibytes = size.trim_end_matches("miB").parse::<u64>()?;
            Ok(Size::Mebibytes(mebibytes))
        } else if size.ends_with("gB") {
            let gigabytes = size.trim_end_matches("gB").parse::<u64>()?;
            Ok(Size::Gigabytes(gigabytes))
        } else if size.ends_with("giB") {
            let gibibytes = size.trim_end_matches("giB").parse::<u64>()?;
            Ok(Size::Gibibytes(gibibytes))
        } else {
            let bytes = size.parse::<u64>()?;
            Ok(Size::Bytes(bytes))
        }
    }

    pub fn to_bytes(&self) -> u64 {
        match self {
            Size::Bytes(bytes) => *bytes,
            Size::Kilobytes(kilobytes) => *kilobytes * 1000,
            Size::Kibibytes(kibibytes) => *kibibytes * 1024,
            Size::Megabytes(megabytes) => *megabytes * 1000 * 1000,
            Size::Mebibytes(mebibytes) => *mebibytes * 1024 * 1024,
            Size::Gigabytes(gigabytes) => *gigabytes * 1000 * 1000 * 1000,
            Size::Gibibytes(gibibytes) => *gibibytes * 1024 * 1024 * 1024,
        }
    }
}
