#[derive(Debug)]
pub enum Error {
    HTTPError,
    SatispayError { code: i64, message: String },
    GenericError,
}
