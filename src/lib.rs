mod daily_closure;
pub mod payment;
mod person;
mod utils;

pub use daily_closure::*;
use payment::*;
pub use person::*;
use rsa;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
pub use utils::SatispayError;
use utils::*;

pub struct Satispay {
    private_key: rsa::RSAPrivateKey,
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
            private_key: rsa::RSAPrivateKey::from_pkcs1(
                base64::decode(&privkey_file)
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
    ) -> ureq::Response {
        let body = body
            .map(|x| serde_json::to_string(&x).unwrap())
            .unwrap_or("".to_string());
        let digest = format!(
            "SHA256={}",
            base64::encode(Sha256::digest(&body.as_bytes()))
        );
        request.set("Digest", &digest);
        // Check the mandatory headers
        if !request.has("host") {
            request.set("Host", "authservices.satispay.com");
        }
        if !request.has("date") {
            request.set("Date", &chrono::Utc::now().to_rfc2822());
        }
        let headers = ["digest", "host", "date"]
            .iter()
            .map(|x| format!("{}: {}", *x, request.header(x).unwrap()))
            .collect::<Vec<String>>()
            .join("\n");
        let string = format!(
            "(request-target): {} {}{}\n{}",
            request.get_method().to_lowercase(),
            request.get_path().unwrap(),
            request.get_query().unwrap(),
            headers
        );
        let string_digest = Sha256::digest(string.as_bytes());
        let padding = rsa::PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256));
        let signature = base64::encode(self.private_key.sign(padding, &string_digest).unwrap());
        let signature_header = format!("keyId=\"{}\", algorithm=\"rsa-sha256\", headers=\"(request-target) digest host date\", signature=\"{}\"", self.key_id, signature);
        request.auth_kind("Signature", &signature_header);
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
            .into_string()
            .map_err(|_| Error::HTTPError)?;
        let response: std::collections::HashMap<String, &str> =
            serde_json::from_str(&response_json)
                .map_err(|_| errorize(serde_json::from_str::<SatispayError>(&response_json)))?;
        Ok(response.get("id").unwrap().to_string())
    }
}
