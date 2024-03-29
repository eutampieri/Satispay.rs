mod daily_closure;
pub mod error;
pub mod payment;
mod person;
mod utils;

use base64::{engine::general_purpose, Engine as _};
pub use daily_closure::*;
use error::Error;
use payment::*;
pub use person::*;
use rsa::{self, pkcs1::DecodeRsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use utils::*;

#[derive(Serialize)]
struct Update {
    action: Action,
}

pub struct Satispay {
    private_key: rsa::RsaPrivateKey,
    key_id: String,
}

impl Satispay {
    /// Load a Satispay instance from the private key and the keyId
    pub fn from_files(key_file: &str, key_id_file: &str) -> Self {
        let key_id_file: HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(key_id_file).unwrap()).unwrap();
        let privkey_file = std::fs::read_to_string(key_file)
            .unwrap()
            .split("\n")
            .filter(|x| x.len() != 0)
            .filter(|x| x.chars().nth(0).unwrap() != '-')
            .collect::<Vec<&str>>()
            .join("");
        Self {
            private_key: rsa::RsaPrivateKey::from_pkcs1_der(
                general_purpose::STANDARD_NO_PAD
                    .decode(&privkey_file)
                    .expect("Cannot load private key")
                    .as_slice(),
            )
            .expect("Cannot parse private key"),
            key_id: key_id_file.get("key_id").unwrap().clone(),
        }
    }
    /// Internal function used to sign the request
    fn sign_and_send<T: Serialize>(
        &self,
        mut request: ureq::Request,
        body: Option<T>,
    ) -> Result<ureq::Response, ureq::Error> {
        let body = body
            .map(|x| serde_json::to_string(&x).unwrap())
            .unwrap_or("".to_string());
        let digest = format!(
            "SHA256={}",
            general_purpose::STANDARD_NO_PAD.encode(Sha256::digest(&body.as_bytes()))
        );
        request = request.set("Digest", &digest);
        // Check the mandatory headers
        if !request.has("host") {
            request = request.set("Host", "authservices.satispay.com");
        }
        if !request.has("date") {
            request = request.set("Date", &chrono::Utc::now().to_rfc2822());
        }
        let headers = ["digest", "host", "date"]
            .iter()
            .map(|x| format!("{}: {}", *x, request.header(x).unwrap()))
            .collect::<Vec<String>>()
            .join("\n");
        let query = request
            .request_url()?
            .query_pairs()
            .into_iter()
            .map(|x| format!("{}={}", x.0, x.1))
            .collect::<Vec<_>>()
            .join("&");
        let string = format!(
            "(request-target): {} {}{}\n{}",
            request.method().to_lowercase(),
            request.request_url()?.path(),
            if query.len() > 0 {
                format!("?{}", query)
            } else {
                "".to_owned()
            },
            headers
        );
        let string_digest = Sha256::digest(string.as_bytes());
        let padding = rsa::pkcs1v15::Pkcs1v15Sign::new::<sha2::Sha256>();
        let signature = general_purpose::STANDARD_NO_PAD
            .encode(self.private_key.sign(padding, &string_digest).unwrap());
        let signature_header = format!("keyId=\"{}\", algorithm=\"rsa-sha256\", headers=\"(request-target) digest host date\", signature=\"{}\"", self.key_id, signature);
        request = request.set("Authorization", &format!("Signature {}", signature_header));
        if body != "" {
            request = request.set("Content-Type", "application/json");
        }
        request.send_string(&body)
    }
    /// API to retrieve the list of payments for a specific shop. The shop is automatically filtered based on the KeyID used in the authorisation header.
    pub fn get_payments_list(
        &self,
        status: Option<PaymentStatus>,
        limit: Option<i32>,
        starting_after: Option<String>,
        starting_after_timestamp: Option<u64>,
    ) -> Result<Vec<Payment>, Error> {
        #[derive(Deserialize)]
        struct Response {
            //has_more: bool,
            data: Vec<Payment>,
        }

        let mut query: String = "".to_owned();
        {
            push_to_query(&mut query, "status", status);
            push_to_query(&mut query, "limit", limit.map(|x| format!("{}", x)));
            push_to_query(
                &mut query,
                "starting_after_timestamp",
                starting_after_timestamp.map(|x| format!("{}", x)),
            );
            push_to_query(&mut query, "starting_after", starting_after);
        }
        let response_json = &self
            .sign_and_send::<String>(
                ureq::get(&format!(
                    "https://authservices.satispay.com/g_business/v1/payments{}",
                    query
                )),
                None,
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: Response = serde_json::from_str(&response_json)
            .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response.data)
    }
    /// API to retrieve the detail of a specific payment
    pub fn get_payment(&self, id: &str) -> Result<Payment, Error> {
        let response_json = &self
            .sign_and_send::<String>(
                ureq::get(&format!(
                    "https://authservices.satispay.com/g_business/v1/payments/{}",
                    id
                )),
                None,
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: Payment = serde_json::from_str(&response_json)
            .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response)
    }
    /// API to retrieve a customer uid from the phone number
    pub fn retrieve_customer(&self, phone_number: &str) -> Result<String, Error> {
        let response_json = &self
            .sign_and_send::<String>(
                ureq::get(&format!(
                    "https://authservices.satispay.com/g_business/v1/consumers/{}",
                    phone_number
                )),
                None,
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: std::collections::HashMap<String, &str> =
            serde_json::from_str(&response_json)
                .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response.get("id").unwrap().to_string())
    }
    /// API to retrieve a customer uid from the phone number
    pub fn update_payment(&self, id: &str, action: Action) -> Result<Payment, Error> {
        let response_json = &self
            .sign_and_send::<Update>(
                ureq::put(&format!(
                    "https://authservices.satispay.com/g_business/v1/payments/{}",
                    id
                )),
                Some(Update { action }),
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: Payment = serde_json::from_str(&response_json)
            .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response)
    }
    /// API to retrieve shop daily closure
    pub fn retrieve_daily_closure(
        &self,
        date: chrono::Date<chrono::Utc>,
    ) -> Result<(daily_closure::DailyClosure, PDF), Error> {
        #[derive(Deserialize)]
        struct Response {
            shop_daily_closure: daily_closure::DailyClosure,
            pdf: PDF,
        }

        let response_json = &self
            .sign_and_send::<String>(
                ureq::get(&format!(
                    "https://authservices.satispay.com/g_business/v1/daily_closure/{}?generate_pdf=true",
                    date.format("%Y%m%d").to_string()
                )),
                None,
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: Response = serde_json::from_str(&response_json)
            .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok((response.shop_daily_closure, response.pdf))
    }
    /// API to create a payment
    pub fn create_payment(&self, payment: NewPayment) -> Result<Payment, Error> {
        let response_json = &self
            .sign_and_send::<NewPayment>(
                ureq::post(&format!(
                    "https://authservices.satispay.com/g_business/v1/payments",
                )),
                Some(payment),
            )
            .map_err(|_| Error::HTTPError)?
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        println!("{}", response_json);
        let response: Payment = serde_json::from_str(&response_json)
            .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let json = "{\"id\":\"id\",\"code_identifier\":\"code\",\"type\":\"TO_BUSINESS\",\"amount_unit\":700,\"currency\":\"EUR\",\"status\":\"PENDING\",\"expired\":false,\"metadata\":{},\"sender\":{\"type\":\"CONSUMER\"},\"receiver\":{\"id\":\"1234\",\"type\":\"SHOP\"},\"insert_date\":\"2020-08-06T08:08:29.706Z\",\"expire_date\":\"2020-08-06T10:08:29.700Z\",\"description\":\"session=01EF1D0NGQ3J12C5VDW2GD3PSP\",\"flow\":\"CHARGE\",\"external_code\":\"session=01EF1D0NGQ3J12C5VDW2GD3PSP\"}";
        let _: Payment = serde_json::from_str(json).unwrap();
        //println!("{:?}", data);
    }
}
