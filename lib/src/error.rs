use std::fmt;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct InMemoryDataError;

impl std::error::Error for InMemoryDataError {
    fn description(&self) -> &str {
        return "InMemoryDataError";
    }
}

impl fmt::Display for InMemoryDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for InMemoryDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct TimeStringUnknownOperator;

impl std::error::Error for TimeStringUnknownOperator {
    fn description(&self) -> &str {
        return "Unknonw operator";
    }
}

impl fmt::Display for TimeStringUnknownOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for TimeStringUnknownOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
