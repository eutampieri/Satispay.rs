#[allow(non_camel_case_types)]
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
pub enum Type {
    CONSUMER,
    SHOP,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Person {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub person_type: Type,
    pub name: Option<String>,
}
