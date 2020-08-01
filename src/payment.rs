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
    id: String,
    /// Type of payment Rename to type
    #[serde(rename = "type")]
    payment_type: Type,
    /// Amount of the payment in cents
    amount_unit: i64,
    /// Currency of the payment
    currency: String,
    /// Status of the payment
    status: PaymentStatus,
    /// If true, the device making the request is responsible for the final status reached by the payment
    status_ownership: bool,
    /// If true, the payment is expired
    expired: bool,
    /// Additional metadata of the payment
    sender: super::Person,
    /// The receiver actor of the payment
    reciever: super::Person,
    daily_closure: DailyClosure,
    insert_date: chrono::DateTime<chrono::Utc>,
    expire_date: chrono::DateTime<chrono::Utc>,
    external_code: String,
}
