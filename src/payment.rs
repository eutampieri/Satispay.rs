use serde::{Deserialize, Serialize};
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize)]
pub enum Type {
    TO_BUSINESS,
    REFUND_TO_BUSINESS,
}
#[derive(Deserialize, Serialize)]
pub enum PaymentStatus {
    PENDING,
    ACCEPTED,
    CANCELED,
}

impl<'a> From<PaymentStatus> for String {
    fn from(status: PaymentStatus) -> Self {
        match status {
            PaymentStatus::ACCEPTED => "ACCEPTED",
            PaymentStatus::CANCELED => "CANCELED",
            PaymentStatus::PENDING => "PENDING",
        }
        .to_owned()
    }
}

#[derive(Deserialize, Serialize)]
pub struct DailyClosure {
    id: String,
    date: chrono::DateTime<chrono::Utc>,
}
#[derive(Deserialize, Serialize)]
pub struct Payment {
    /// Unique ID of the payment
    pub id: String,
    /// Generated QR code identifier
    #[serde(default)]
    pub code_identifier: Option<String>,
    /// Type of payment Rename to type
    #[serde(rename = "type")]
    pub payment_type: Type,
    /// Amount of the payment in cents
    pub amount_unit: i64,
    /// Currency of the payment
    pub currency: String,
    /// Status of the payment
    pub status: PaymentStatus,
    /// If true, the device making the request is responsible for the final status reached by the payment
    #[serde(default)]
    pub status_ownership: Option<bool>,
    /// If true, the payment is expired
    pub expired: bool,
    /// Additional metadata of the payment
    pub sender: super::Person,
    /// The receiver actor of the payment
    pub reciever: super::Person,
    /// The daily closure of the payment
    #[serde(default)]
    pub daily_closure: Option<DailyClosure>,
    /// Timestamp of payment insertion
    pub insert_date: chrono::DateTime<chrono::Utc>,
    /// Timestamp of payment expiration
    pub expire_date: chrono::DateTime<chrono::Utc>,
    /// Order ID or payment external identifier
    pub external_code: String,
}
