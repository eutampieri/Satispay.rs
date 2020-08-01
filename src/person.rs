#[allow(non_camel_case_types)]
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub enum Type {
    CONSUMER,
    SHOP,
}
#[derive(Deserialize, Serialize)]
pub struct Person {
    pub id: String,
    #[serde(rename = "type")]
    pub person_type: Type,
    pub name: Option<String>,
}
