use std::collections::BTreeMap;
use serde::{Deserialize, de::{Deserializer, DeserializeOwned, Error as DeError}};
use serde_json::from_str;


pub fn nested<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Nested<T> {
        Map(BTreeMap<String, T>),
        Vec(Vec<T>),
        Json(String),
    }

    match Nested::<T>::deserialize(deserializer)? {
        Nested::Map(m) => Ok(
            m
                .into_iter()
                .filter_map(|(k, v)| k.parse::<usize>().ok().map(|n| (n, v)))
                .collect::<BTreeMap<usize, T>>()
                .into_values()
                .collect()
        ),
        Nested::Vec(v) => Ok(v),
        Nested::Json(s) => from_str(&s).map_err(DeError::custom),
    }
}
