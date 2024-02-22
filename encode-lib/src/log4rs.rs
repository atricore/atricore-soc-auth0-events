use crate::{Encode, EncodeError, EncodeSettings};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct Log4rsEncoder {
    pub settings: Log4rsEncoderSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log4rsEncoderSettings {
    pub enable: Option<bool>,
    pub logger_name: Box<str>,
}

impl Log4rsEncoder {
    pub fn new(s: Log4rsEncoderSettings) -> Log4rsEncoder {
        Log4rsEncoder { settings: s }
    }
}

impl EncodeSettings for Log4rsEncoderSettings {}

impl Encode for Log4rsEncoder {
    fn encode(&self, data: &Value) -> Result<(), EncodeError> {
        // data is an array of alerts:
        //
        if let Some(arr) = data.as_array() {
            for alert in arr {
                //
                match alert.as_array() {
                    Some(a) => {
                        for ad in a {
                            let auth0alert = json!({"auth0alert": ad, "event": "auth0"});
                            serde_json::to_string(&auth0alert)
                                .map(|v| {
                                    info!(target: &self.settings.logger_name, "{}", v);
                                })
                                .map_err(|e| EncodeError::new(&e.to_string()))?;
                        }
                    }

                    None => {
                        let auth0alert = json!({"auth0alert": alert, "event": "auth0"});
                        serde_json::to_string(&auth0alert)
                            .map(|v| {
                                info!(target: &self.settings.logger_name, "{}", v);
                            })
                            .map_err(|e| EncodeError::new(&e.to_string()))?;
                    }
                };
            }
            Ok(())
        } else {
            Err(EncodeError::new(
                "data is not a JSON array. Make sure to configure Auth0 log stream to JSON Array",
            ))
        }
    }

    fn name(&self) -> String {
        "log4rs-encode".to_string()
    }
}
