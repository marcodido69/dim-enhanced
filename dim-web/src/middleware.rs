use axum::extract::State;
use dim_core::errors::DimError;
use dim_database::DbConnection;

use crate::DimErrorWrapper;

/// Rutas que no requieren token:
/// - todo lo que no sea API (SPA, estaticos, imagenes, websocket),
/// - login / registro / chequeo de admin,
/// - los segmentos del reproductor DASH (dash.js no manda cabeceras).
///
/// El manifiesto virtual (`/api/v1/stream/<id>/manifest`) si requiere usuario,
/// por eso se excluye de la excepcion de streaming.
fn is_public_path(path: &str) -> bool {
    if !path.starts_with("/api/") {
        return true;
    }

    if matches!(
        path,
        "/api/v1/auth/login" | "/api/v1/auth/register" | "/api/v1/auth/admin_exists"
    ) {
        return true;
    }

    path.starts_with("/api/v1/stream/") && !path.ends_with("/manifest")
}

pub async fn verify_cookie_token<B>(
    State(conn): State<DbConnection>,
    mut req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, DimErrorWrapper> {
    if is_public_path(req.uri().path()) {
        return Ok(next.run(req).await);
    }

    match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(token) => {
            let mut tx = match conn.read().begin().await {
                Ok(tx) => tx,
                Err(_) => {
                    return Err(DimErrorWrapper(DimError::DatabaseError {
                        description: String::from("Failed to start transaction"),
                    }))
                }
            };
            let id = dim_database::user::Login::verify_cookie(token.to_str().unwrap().to_string())
                .map_err(|e| DimError::CookieError(e))
                .map_err(|e| DimErrorWrapper(e))?;

            let current_user = dim_database::user::User::get_by_id(&mut tx, id)
                .await
                .map_err(|_| DimError::UserNotFound)
                .map_err(|e| DimErrorWrapper(e))?;

            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        None => Err(DimErrorWrapper(DimError::NoToken)),
    }
}
