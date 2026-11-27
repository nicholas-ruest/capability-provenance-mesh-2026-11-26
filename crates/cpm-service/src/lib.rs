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
use cpm_upstream_runtime::ProbeKind;
use serde::Serialize;
use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::time::timeout;
use tower::ServiceBuilder;
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
    autogenous: cpm_autogenous::Adapter,
    rugraph: cpm_rugraph::Adapter,
    agenticow: cpm_agenticow::Adapter,
    watermark: cpm_watermark::Adapter,
    latentmesh: cpm_latentmesh::Adapter,
    skillspector: cpm_skillspector::Adapter,
    cosign: cpm_cosign::Adapter,
}
impl RealSuite {
    #[must_use]
    pub fn from_harness_root(root: &FsPath) -> Self {
        let pair =
            |success: &str, failure: &str| (Some(root.join(success)), Some(root.join(failure)));
        let (a, b) = pair("autogenous-success.sh", "autogenous-failure.sh");
        let autogenous = cpm_autogenous::Adapter::new(a, b);
        let (a, b) = pair("rugraph-success.sh", "rugraph-failure.sh");
        let rugraph = cpm_rugraph::Adapter::new(a, b);
        let (a, b) = pair("agenticow-success.sh", "agenticow-failure.sh");
        let agenticow = cpm_agenticow::Adapter::new(a, b);
        let (a, b) = pair("watermark-success.sh", "watermark-failure.sh");
        let watermark = cpm_watermark::Adapter::new(a, b);
        let (a, b) = pair("latentmesh-success.sh", "latentmesh-failure.sh");
        let latentmesh = cpm_latentmesh::Adapter::new(a, b);
        let (a, b) = pair("skillspector-success.sh", "skillspector-failure.sh");
        let skillspector = cpm_skillspector::Adapter::new(a, b);
        let (a, b) = pair("cosign-success.sh", "cosign-failure.sh");
        let cosign = cpm_cosign::Adapter::new(a, b);
        Self {
            autogenous,
            rugraph,
            agenticow,
            watermark,
            latentmesh,
            skillspector,
            cosign,
        }
    }
}
impl UpstreamSuite for RealSuite {
    fn execute_all(&self, denial_probe: bool) -> Result<Vec<String>, String> {
        let kind = if denial_probe {
            ProbeKind::Failure
        } else {
            ProbeKind::Success
        };
        let receipts = [
            self.autogenous.execute(kind),
            self.rugraph.execute(kind),
            self.agenticow.execute(kind),
            self.watermark.execute(kind),
            self.latentmesh.execute(kind),
            self.skillspector.execute(kind),
            self.cosign.execute(kind),
        ];
        receipts
            .into_iter()
            .map(|r| {
                r.map(|x| format!("{}@{}:{}", x.repository, x.revision, x.output_sha256))
                    .map_err(|e| e.to_string())
            })
            .collect()
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
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/v1/intakes", post(create_intake))
        .route("/api/v1/registry/{plugin}/{version}", get(get_registry))
        .route("/api/v1/audit/{correlation}", get(get_audit))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(governance)));
    Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/readyz", get(|| async { StatusCode::OK }))
        .merge(protected)
        .with_state(state)
}
async fn governance(mut request: Request, next: Next) -> Response {
    let correlation = request
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 96)
        .map_or_else(|| Uuid::new_v4().to_string(), ToOwned::to_owned);
    let span = info_span!("http_request",correlation_id=%correlation);
    let _guard = span.enter();
    if request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| !v.starts_with("Bearer "))
    {
        return error(
            StatusCode::UNAUTHORIZED,
            "authn_required",
            "valid bearer token required",
            correlation,
        );
    }
    if request
        .headers()
        .get("x-cpm-role")
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| !matches!(v, "reviewer" | "admin"))
    {
        return error(
            StatusCode::FORBIDDEN,
            "authz_denied",
            "reviewer role required",
            correlation,
        );
    }
    if request
        .headers()
        .get("x-cpm-policy")
        .is_some_and(|v| v == "deny")
    {
        return error(
            StatusCode::FORBIDDEN,
            "policy_denied",
            "policy denied before effects",
            correlation,
        );
    }
    if request.method() == axum::http::Method::POST
        && request
            .headers()
            .get(header::CONTENT_TYPE)
            .is_none_or(|v| v != "application/json")
    {
        return error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "validation_failed",
            "application/json required",
            correlation,
        );
    }
    request.extensions_mut().insert(correlation.clone());
    match timeout(Duration::from_secs(10), next.run(request)).await {
        Ok(mut response) => {
            if let Ok(value) = HeaderValue::from_str(&correlation) {
                response.headers_mut().insert("x-correlation-id", value);
            }
            response.headers_mut().insert(
                "x-cpm-middleware-order",
                HeaderValue::from_static(
                    "correlation,observability,error,timeout,authn,validation,authz,policy",
                ),
            );
            response
        }
        Err(_) => error(
            StatusCode::REQUEST_TIMEOUT,
            "timeout",
            "request deadline exceeded",
            correlation,
        ),
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
