use serde::{Deserialize, Serialize};
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Eq, PartialEq)]
pub enum Type {
    TO_BUSINESS,
    REFUND_TO_BUSINESS,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Eq, PartialEq)]
pub enum Action {
    /// to confirm a pending payment created by the users
    ACCEPT,
    /// to cancel a pending payment
    CANCEL,
    /// to request a payment to be either canceled if still pending or refunded if already accepted
    CANCEL_OR_REFUND,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Eq, PartialEq)]
pub enum Flow {
    MATCH_CODE,
    MATCH_USER,
    REFUND,
    PRE_AUTHORIZED,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
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
    pub receiver: super::Person,
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

#[derive(Serialize)]
pub struct NewPayment {
    /// The flow of the payment
    pub flow: Flow,
    /// Amount of the payment in cents
    pub amount_unit: i32,
    /// Pre-Authorized token id (required with the PRE_AUTHORIZED flow only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_authorized_payments_token: Option<String>,
    /// Unique ID of the payment to refund (required with the REFUND flow only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_payment_uid: Option<String>,
    /// Currency of the payment
    pub currency: String,
    /// The expiration date of the payment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Order ID or payment external identifier. Max length allowed is 50 chars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    /// The url that will be called with an http GET request when the Payment changes state. When url is called a Get payment details can be called to know the new Payment status. Note that {uuid} will be replaced with the Payment ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Unique ID of the consumer that has to accept the payment. To retrieve the customer uid use the Retrive customer API (required with the MATCH_USER flow only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_uid: Option<String>,
}
