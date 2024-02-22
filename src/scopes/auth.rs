use crate::error::HttpError;
use crate::RequireAuth;
use crate::Token;
use a3c_soc_auth0_events_encode_lib::Encode;
use actix_web::{web, web::Data, HttpRequest, HttpResponse, Scope};
use log::{debug, error};
use serde_json::Value;

pub fn auth_scope(token: String) -> Scope {
    web::scope("/a3c/auth0").route(
        "/events",
        web::post()
            .to(handle_auth0_event)
            .wrap(RequireAuth::allowed_tokens(vec![Token::new(token)])),
    )
}

pub async fn handle_auth0_event(item: web::Json<Value>, req: HttpRequest) -> HttpResponse {
    debug!("hadle auth0 event");

    // JSON must be an array
    let Some(_) = &item.0.as_array() else {
        error!("JSON data is not an array. Make sure to configure Auth0 log stream content format to JSON Array");
        return HttpError::new("JSON data is not an array. Make sure to configure Auth0 log stream content format to JSON Array", 500).into_http_response();
    };

    // Chekc if an Encode is in our APP DATA
    let r = match req.app_data::<Data<dyn Encode>>() {
        Some(encoder) => {
            debug!("using encoder {}", encoder.name());
            match encoder.encode(&item.0) {
                Ok(()) => HttpResponse::Ok().json("SUCCESS"),
                Err(e) => {
                    let error_message = format!("Error decoding value: {:?}", e);
                    HttpError::new(error_message, 500).into_http_response()
                }
            }
        }
        None => {
            error!("No encoder found in app_data");
            HttpError::new("No encoder found in app_data", 500).into_http_response()
        }
    };
    r
}
