use serde::Deserialize;
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Eq, PartialEq)]
pub enum Type {
    CONSUMER,
    SHOP,
}

#[derive(Deserialize, Debug)]
/// The receipt of the daily closure
pub struct PDF {
    /// The pre-signed url to the daily closure pdf
    pub url: String,
    /// The expiration date of the pre-signed url
    pub expiration: chrono::DateTime<chrono::Utc>,
    /// The time to live of the pre-signed url in seconds
    pub expire_in_sec: u32,
    /// The bucket in which the pdf is stored
    pub bucket: String,
    /// The key of the pdf,
    pub key: String,
    /// The http method that can be invoked on pre-signed url
    pub http_method: String,
}
#[derive(Deserialize, Debug)]
pub struct DailyClosure {
    /// Unique ID of the daily closure
    pub id: String,
    /// Type of the daily closure
    #[serde(rename = "type")]
    pub closure_type: Type,
    /// Unique ID of the shop
    pub customer_uid: String,
    /// The daily closure amount of the whole shop
    pub amount_unit: i64,
    /// The currency of the daily closure
    pub currency: String,
}
