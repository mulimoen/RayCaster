use fmt;
use std::error::Error;
use std::{io, num};

#[derive(Debug)]
pub enum VTKparseError {
    Io(io::Error),
    FileFormat(String),
    NotImplemented(String),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
    UnknownFormat(String),
    WrongFormat(String),
}

impl fmt::Display for VTKparseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VTKparseError::Io(ref err) => write!(f, "IO error: {}", err),
            VTKparseError::ParseInt(ref err) => write!(f, "Int conversion: {}", err),
            VTKparseError::ParseFloat(ref err) => write!(f, "Float conversion: {}", err),
            VTKparseError::FileFormat(ref err) => write!(f, "FF error: {}", err),
            VTKparseError::NotImplemented(ref err) => write!(f, "{} is not yet implemented", err),
            VTKparseError::UnknownFormat(ref err) => write!(f, "Format is not known: {}", err),
            VTKparseError::WrongFormat(ref err) => write!(f, "Format was not recognized: {}", err),
        }
    }
}

impl Error for VTKparseError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            VTKparseError::Io(ref err) => Some(err),
            VTKparseError::ParseInt(ref err) => Some(err),
            VTKparseError::ParseFloat(ref err) => Some(err),
            VTKparseError::FileFormat(_)
            | VTKparseError::NotImplemented(_)
            | VTKparseError::UnknownFormat(_)
            | VTKparseError::WrongFormat(_) => None,
        }
    }
}

impl From<num::ParseIntError> for VTKparseError {
    fn from(err: num::ParseIntError) -> VTKparseError {
        VTKparseError::ParseInt(err)
    }
}

impl From<num::ParseFloatError> for VTKparseError {
    fn from(err: num::ParseFloatError) -> VTKparseError {
        VTKparseError::ParseFloat(err)
    }
}

impl From<io::Error> for VTKparseError {
    fn from(err: io::Error) -> VTKparseError {
        VTKparseError::Io(err)
    }
}
