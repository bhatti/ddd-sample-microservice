pub const DATE_FMT: &str = "%Y-%m-%dT%H:%M:%S%.f";

pub mod serializer {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Error;
    use crate::utils::date::DATE_FMT;

    pub fn serialize<S: Serializer>(time: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error> {
        time_to_json(*time).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDateTime, D::Error> {
        let str_time: String = Deserialize::deserialize(deserializer)?;
        let time = NaiveDateTime::parse_from_str(&str_time, DATE_FMT).map_err(D::Error::custom)?;
        Ok(time)
    }

    fn time_to_json(t: NaiveDateTime) -> String {
        DateTime::<Utc>::from_utc(t, Utc).to_rfc3339()
    }
}
