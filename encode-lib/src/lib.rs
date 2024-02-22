pub mod log4rs;

use derive_more::{Display, Error};
use log4rs::Log4rsEncoderSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait Encode {
    fn encode(&self, data: &Value) -> Result<(), EncodeError>;
    fn name(&self) -> String;
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Error in encoder: {}", msg)]
pub struct EncodeError {
    msg: String,
}

impl EncodeError {
    pub fn new(message: &str) -> EncodeError {
        EncodeError {
            msg: message.to_string(),
        }
    }
}

pub trait EncodeSettings {}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum EncoderType {
    #[serde(rename = "log4rs")]
    Log4rs(Log4rsEncoderSettings),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncodersSettings {
    pub encoders: Vec<EncoderType>,
}
