pub enum Error {
    InMemoryDataError,
    TimeStringUnknownOperator,
    UndefinedExtraData,
    FromStringError,
    GenericError,
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    ParseIntError(std::num::ParseIntError),
    SerdeYamlError(serde_yaml::Error),
    ReqwestError(reqwest::Error),
    RegexError(regex::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InMemoryDataError, Self::InMemoryDataError) => true,
            (Self::TimeStringUnknownOperator, Self::TimeStringUnknownOperator) => true,
            (Self::FromStringError, Self::FromStringError) => true,
            (Self::UndefinedExtraData, Self::UndefinedExtraData) => true,
            _ => false,
        }
    }
}

impl Error {
    fn as_str(&self) -> &str {
        match self {
            Self::InMemoryDataError => "InMemoryDataError",
            Self::TimeStringUnknownOperator => "Unknown Operator",
            Self::UndefinedExtraData => "UndefinedExtraData",
            Self::FromStringError => "FromStrError",
            _ => "NoString",
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::SerdeYamlError(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error::Utf8Error(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::RegexError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::ParseIntError(error)
    }
}
