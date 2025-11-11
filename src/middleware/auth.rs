use crate::{
    config::ENV,
    error::AppError,
    middleware::auth::proto::{
        ValidateAccessTokenRequest, token_service_client::TokenServiceClient,
    },
    models::user::UserId,
    util::gcloud,
};
use axum::{extract::Request, middleware::Next, response::Response};
use envmode::EnvMode;
use tonic::{
    metadata::MetadataValue,
    transport::{Channel, ClientTlsConfig},
};

pub mod proto {
    tonic::include_proto!("token");
}

pub async fn access_token(mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::bad_request("missing authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::bad_request("invalid authorization header"))?;
    if token.is_empty() {
        return Err(AppError::bad_request("token cannot be emtpy"));
    }
    let idtoken = gcloud::idtoken().await?;

    let domain = if EnvMode::is_prd(&ENV.environment) {
        &*ENV.grpc_prd_domain
    } else {
        &*ENV.grpc_dev_domain
    };
    let audience = format!("https://{}", domain);

    let tls_config = ClientTlsConfig::new().with_webpki_roots();
    let channel = Channel::from_shared(audience)
        .map_err(AppError::from_generic_error)?
        .tls_config(tls_config)
        .map_err(AppError::from_generic_error)?
        .connect()
        .await
        .map_err(AppError::from_generic_error)?;

    let mut client = TokenServiceClient::new(channel);

    let mut request = tonic::Request::new(ValidateAccessTokenRequest {
        token: token.to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::try_from(format!("Bearer {}", idtoken))
            .map_err(AppError::from_generic_error)?,
    );

    let response = client
        .validate_user_access_token(request)
        .await
        .map_err(|e| match e.code() {
            tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => {
                AppError::unauthorized_with_source("invalid access token", e)
            }
            _ => AppError::from_generic_error(e),
        })?;
    let claims = response
        .into_inner()
        .claims
        .ok_or_else(|| AppError::unauthorized("missing token claims"))?;

    req.extensions_mut().insert(UserId(claims.sub));
    Ok(next.run(req).await)
}
