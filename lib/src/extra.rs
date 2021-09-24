use super::error::Error;
use super::serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// TODO this is a rather inefficient way
/// to store extra data
/// but it works well with the current serde piepeline
/// Maybe implement a proper system in the future?
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ExtraData {
    data: HashMap<String, String>,
}

impl ExtraData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn put<T>(
        &mut self,
        name: &str,
        data: &T,
        serialize: fn(&T) -> Result<String, Error>,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        self.data.insert(name.into(), serialize(data)?);
        Ok(())
    }

    pub fn get<T>(
        &mut self,
        name: &str,
        deserialize: fn(&str) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let data = match self.data.get(name) {
            Some(d) => d,
            _ => return Err(Error::UndefinedExtraData),
        };

        deserialize(data)
    }

    pub fn serialize<T>(data: &T) -> Result<String, Error>
    where
        T: Serialize,
    {
        Ok(serde_yaml::to_string(&data)?)
    }

    pub fn deserialize<T>(s: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        Ok(serde_yaml::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct TestData {
        value1: u32,
        value2: u32,
    }

    #[test]
    fn it_should_serialize_and_deserialize_test_data() {
        let test = TestData {
            value1: 100,
            value2: 200,
        };

        let mut extra = ExtraData::new();

        extra.put("Test", &test, ExtraData::serialize).unwrap();

        let deser = extra
            .get::<TestData>("Test", ExtraData::deserialize)
            .unwrap();

        assert_eq!(deser, test);
    }

    #[test]
    fn it_should_not_find_invalid_key() {
        let mut extra = ExtraData::new();
        let res = extra.get::<TestData>("Test", ExtraData::deserialize);
        assert_eq!(res, Err(Error::UndefinedExtraData));
    }

    #[test]
    #[should_panic]
    fn it_should_not_cast_to_bad_type() {
        let test = TestData {
            value1: 100,
            value2: 200,
        };

        let mut extra = ExtraData::new();

        extra.put("Test", &test, ExtraData::serialize).unwrap();

        let _ = extra.get::<u32>("Test", ExtraData::deserialize).unwrap();
    }
}
