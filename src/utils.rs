use super::error::Error;
use serde::Deserialize;

pub fn push_to_query<'a, T>(query: &mut String, parameter_name: &str, parameter_value: Option<T>)
where
    T: Into<String>,
{
    if let Some(param) = parameter_value {
        if query == "" {
            *query += "?";
        } else {
            *query += "&";
        }
        *query += parameter_name;
        *query += "=";
        *query += &(param.into());
    }
}

pub fn errorize<E>(result: Result<SatispayError, E>) -> Error {
    if let Ok(x) = result {
        Error::SatispayError {
            code: x.code,
            message: x.message,
        }
    } else {
        Error::GenericError
    }
}

#[derive(Deserialize)]
pub struct SatispayError {
    code: i64,
    message: String,
}
