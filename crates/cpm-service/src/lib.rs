//! Axum composition root and ordered governance middleware.
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns
)]
use axum::{
    Json, Router,
    extract::{Extension, Path, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use cpm_application::{Application, ApplicationError, IntakeRequest, UpstreamSuite};
use cpm_store::{DecisionRecord, DirectoryStore};
use cpm_upstreams::UpstreamSet;
use serde::Serialize;
use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::time::timeout;
use tracing::info_span;
use uuid::Uuid;
#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    code: &'static str,
    message: String,
    correlation_id: String,
}
fn error(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
    correlation: String,
) -> Response {
    (
        status,
        Json(ErrorEnvelope {
            code,
            message: message.into(),
            correlation_id: correlation,
        }),
    )
        .into_response()
}
#[derive(Clone)]
pub struct RealSuite {
    integrations: UpstreamSet,
}
impl RealSuite {
    #[must_use]
    pub fn from_harness_root(root: &FsPath) -> Self {
        let source_root = std::env::var_os("CPM_UPSTREAM_SOURCE_ROOT")
            .map(PathBuf::from)
            .or_else(|| {
                root.join("upstreams.lock.json")
                    .exists()
                    .then(|| PathBuf::from(".upstreams"))
            })
            .or_else(|| {
                root.parent()
                    .and_then(FsPath::parent)
                    .and_then(FsPath::parent)
                    .map(FsPath::to_path_buf)
            });
        Self {
            integrations: UpstreamSet::from_harness_root(root, source_root),
        }
    }
}
impl UpstreamSuite for RealSuite {
    fn execute_all(&self, denial_probe: bool) -> Result<Vec<String>, String> {
        self.integrations
            .execute_all(denial_probe)
            .map(|evidence| {
                evidence
                    .into_iter()
                    .map(|item| {
                        format!(
                            "{}@{}:{}:{}:{}",
                            item.repository,
                            item.revision,
                            item.responsibility,
                            item.output_sha256,
                            item.semantics_sha256
                        )
                    })
                    .collect()
            })
            .map_err(|error| error.to_string())
    }
}
#[derive(Clone)]
pub struct AppState {
    application: Arc<Application<RealSuite>>,
}
impl AppState {
    pub fn open(data: impl Into<PathBuf>, harness: &FsPath) -> Result<Self, cpm_store::StoreError> {
        Ok(Self {
            application: Arc::new(Application::new(
                DirectoryStore::open(data)?,
                RealSuite::from_harness_root(harness),
                100 * 1024 * 1024,
            )),
        })
    }
}
const MIDDLEWARE_ORDER: &str =
    "correlation,observability,authn,authz,policy,validation,timeout,error";
#[derive(Clone, Copy)]
struct GovernanceConfig {
    request_timeout: Duration,
}
impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
        }
    }
}
pub fn router(state: AppState) -> Router {
    router_with_config(state, GovernanceConfig::default())
}
fn router_with_config(state: AppState, config: GovernanceConfig) -> Router {
    let protected = Router::new()
        .route("/api/v1/intakes", post(create_intake))
        .route("/api/v1/registry/{plugin}/{version}", get(get_registry))
        .route("/api/v1/audit/{correlation}", get(get_audit))
        .layer(middleware::from_fn_with_state(config, governance));
    Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/readyz", get(|| async { StatusCode::OK }))
        .merge(protected)
        .with_state(state)
}
fn finalize_response(
    mut response: Response,
    correlation: &str,
    observation: &'static str,
) -> Response {
    if let Ok(value) = HeaderValue::from_str(correlation) {
        response.headers_mut().insert("x-correlation-id", value);
    }
    response.headers_mut().insert(
        "x-cpm-middleware-order",
        HeaderValue::from_static(MIDDLEWARE_ORDER),
    );
    response
        .headers_mut()
        .insert("x-cpm-observation", HeaderValue::from_static(observation));
    tracing::info!(
        correlation_id = correlation,
        outcome = observation,
        "governance_complete"
    );
    response
}
async fn governance(
    State(config): State<GovernanceConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let correlation = request
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 96)
        .map_or_else(|| Uuid::new_v4().to_string(), ToOwned::to_owned);
    let span = info_span!("http_request",correlation_id=%correlation);
    let _guard = span.enter();
    let authenticated = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| !token.trim().is_empty());
    if !authenticated {
        let response = error(
            StatusCode::UNAUTHORIZED,
            "authn_required",
            "valid bearer token required",
            correlation.clone(),
        );
        return finalize_response(response, &correlation, "authn_denied");
    }
    if request
        .headers()
        .get("x-cpm-role")
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| !matches!(v, "reviewer" | "admin"))
    {
        let response = error(
            StatusCode::FORBIDDEN,
            "authz_denied",
            "reviewer role required",
            correlation.clone(),
        );
        return finalize_response(response, &correlation, "authz_denied");
    }
    if request
        .headers()
        .get("x-cpm-policy")
        .is_some_and(|v| v == "deny")
    {
        let response = error(
            StatusCode::FORBIDDEN,
            "policy_denied",
            "policy denied before effects",
            correlation.clone(),
        );
        return finalize_response(response, &correlation, "policy_denied");
    }
    if request.method() == axum::http::Method::POST
        && request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .is_none_or(|value| value.trim() != "application/json")
    {
        let response = error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "validation_failed",
            "application/json required",
            correlation.clone(),
        );
        return finalize_response(response, &correlation, "validation_denied");
    }
    request.extensions_mut().insert(correlation.clone());
    if let Ok(response) = timeout(config.request_timeout, next.run(request)).await {
        finalize_response(response, &correlation, "completed")
    } else {
        let response = error(
            StatusCode::REQUEST_TIMEOUT,
            "timeout",
            "request deadline exceeded",
            correlation.clone(),
        );
        finalize_response(response, &correlation, "timeout")
    }
}
async fn create_intake(
    State(state): State<AppState>,
    Extension(correlation): Extension<String>,
    Json(payload): Json<IntakeRequest>,
) -> Response {
    match state.application.submit(payload) {
        Ok(value) => Json(value).into_response(),
        Err(error_value) => map_app_error(&error_value, correlation),
    }
}

async fn get_registry(
    State(state): State<AppState>,
    Path((plugin, version)): Path<(String, String)>,
    Extension(correlation): Extension<String>,
) -> Response {
    respond_record(state.application.retrieve(&plugin, &version), correlation)
}

async fn get_audit(
    State(state): State<AppState>,
    Path(audit_correlation): Path<String>,
    Extension(correlation): Extension<String>,
) -> Response {
    respond_record(state.application.audit(&audit_correlation), correlation)
}

fn respond_record(
    result: Result<DecisionRecord, ApplicationError>,
    correlation: String,
) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(error_value) => map_app_error(&error_value, correlation),
    }
}

fn map_app_error(error_value: &ApplicationError, correlation: String) -> Response {
    let status = if matches!(
        error_value,
        ApplicationError::Store(cpm_store::StoreError::Missing)
    ) {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };
    error(
        status,
        "application_error",
        error_value.to_string(),
        correlation,
    )
}
#[cfg(test)]
mod middleware_matrix_tests;
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use tower::ServiceExt;
    fn state() -> AppState {
        let root = std::env::temp_dir().join(format!("cpm-service-{}", Uuid::new_v4()));
        AppState::open(root, FsPath::new("/missing")).unwrap()
    }
    #[tokio::test]
    async fn health_is_public() {
        let response = router(state())
            .oneshot(
                HttpRequest::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK)
    }
    #[tokio::test]
    async fn authn_denies_before_handler() {
        let response = router(state())
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/api/v1/intakes")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
    }
}
