use actix_cors::Cors;
use actix_web::{
    dev::Server,
    http::header,
    {web::Data, App, HttpServer},
};

use log::{debug, error, info};
use std::sync::Arc;

use a3c_soc_auth0_events_encode_lib::{log4rs::Log4rsEncoder, Encode, EncoderType};
use extractors::auth::RequireAuth;
use settings::Settings;

mod error;
mod extractors;
mod scopes;
mod settings;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    value: String,
}

// Authentication token
impl Token {
    pub fn new(value: String) -> Token {
        return Token {
            value: value.clone(),
        };
    }
}
// This starts the Auth0 collector
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = settings::load_config()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    init_log(settings.log_config.to_owned());
    info!("Using address {}", settings.address);
    debug!("Starting HTTP server ...");

    let e = create_log4rs_encoder(&settings).map_err(|e| {
        error!("{}", &e);
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    create_server(settings.address.clone(), settings.api_token.clone(), e)
        .unwrap()
        .await
}

pub fn create_server(
    address: String,
    token: String,
    encode: impl Encode + Send + Sync + 'static + Clone,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        let cors = Cors::default().allowed_headers(vec![
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ]);

        let encode_arc: Arc<dyn Encode> = Arc::new(encode.clone());
        let encode_data: Data<dyn Encode> = Data::from(encode_arc);
        App::new()
            .wrap(cors)
            .app_data(encode_data)
            .service(scopes::auth::auth_scope(token.clone()))
    })
    .bind(address)?
    .run();
    Ok(server)
}

pub fn create_log4rs_encoder(
    settings: &Settings,
) -> Result<Log4rsEncoder, Box<dyn std::error::Error>> {
    // For now, build a log4rs encoder
    let es = settings.encoders.iter().find_map(|e| match e {
        EncoderType::Log4rs(l) => Some(l),
    });

    // if es is None -> Err
    let log4rs_settings = es.ok_or(anyhow::anyhow!("no log4rs encode configured"))?;
    if !log4rs_settings.enable.unwrap_or(false) {
        Err(anyhow::anyhow!("log4rs encoder not enabled"))?;
    };

    Ok(Log4rsEncoder::new(log4rs_settings.to_owned()))
}

fn init_log(f: Option<String>) {
    let fname = f.unwrap_or("./log4rs.yaml".to_string());

    let _ = log4rs::init_file(&fname, Default::default())
        .map_err(|e| {
            panic!("Cannot initialize log: {} {}", fname, e.to_string());
        })
        .map(|_| ());
}
