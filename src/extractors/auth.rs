use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{http, FromRequest, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use futures_util::FutureExt;
use log::debug;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::error::{ErrorMessage, ErrorResponse, HttpError};
use crate::Token;

pub struct Authenticated(Token);

impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<Token>().cloned();
        let result = match value {
            Some(user) => Ok(Authenticated(user)),
            None => Err(ErrorInternalServerError(HttpError::server_error(
                "Authentication Error",
            ))),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequireAuth {
    pub allowed_tokens: Rc<Vec<Token>>,
}

impl RequireAuth {
    pub fn allowed_tokens(allowed_tokens: Vec<Token>) -> Self {
        RequireAuth {
            allowed_tokens: Rc::new(allowed_tokens),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            allowed_tokens: self.allowed_tokens.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    allowed_tokens: Rc<Vec<Token>>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        debug!("poll_ready");
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        debug!("extracting authn token");
        let token: Option<Token> = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|h| Token::new(h.to_str().unwrap().split_at(7).1.to_string()));

        if token.is_none() {
            debug!("no token found, unahtorized!");
            let json_error = ErrorResponse {
                status: "fail".to_string(),
                message: ErrorMessage::TokenNotProvided.to_string(),
            };
            return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
        }

        let token = token.unwrap();
        let t = token.clone().value;
        debug!("token found {} : verifying ... ", t);
        let allowed_tokens = self.allowed_tokens.clone();
        let srv = Rc::clone(&self.service);

        async move {
            // Check if user's role matches the required role
            if allowed_tokens.contains(&token) {
                req.extensions_mut().insert::<Token>(token);
                let res = srv.call(req).await?;
                debug!("token found {} : allowed ... ", t);
                Ok(res)
            } else {
                debug!("token found {} : unallowed ... ", t);
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: ErrorMessage::PermissionDenied.to_string(),
                };
                Err(ErrorForbidden(json_error))
            }
        }
        .boxed_local()
    }
}
