use super::*;
use axum::{
    body::{Body, to_bytes},
    http::Request as HttpRequest,
    routing::post,
};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::time::sleep;
use tower::ServiceExt;

#[derive(Clone)]
struct EffectProbe {
    calls: Arc<AtomicUsize>,
    marker: PathBuf,
    delay: Duration,
}
impl EffectProbe {
    fn new(delay: Duration) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            marker: std::env::temp_dir().join(format!("cpm-effect-{}", Uuid::new_v4())),
            delay,
        }
    }
    fn assert_untouched(&self) {
        assert_eq!(self.calls.load(Ordering::SeqCst), 0, "upstream effect ran");
        assert!(!self.marker.exists(), "durable effect was written");
    }
}
async fn effect_handler(State(probe): State<EffectProbe>) -> StatusCode {
    sleep(probe.delay).await;
    probe.calls.fetch_add(1, Ordering::SeqCst);
    std::fs::write(&probe.marker, b"effect").expect("write effect marker");
    StatusCode::OK
}
fn probe_router(probe: EffectProbe, request_timeout: Duration) -> Router {
    Router::new()
        .route("/probe", post(effect_handler))
        .layer(middleware::from_fn_with_state(
            GovernanceConfig { request_timeout },
            governance,
        ))
        .with_state(probe)
}
fn request(
    auth: Option<&str>,
    role: Option<&str>,
    policy: Option<&str>,
    content_type: Option<&str>,
    correlation: &str,
) -> HttpRequest<Body> {
    let mut builder = HttpRequest::builder()
        .method("POST")
        .uri("/probe")
        .header("x-correlation-id", correlation);
    for (name, value) in [
        ("authorization", auth),
        ("x-cpm-role", role),
        ("x-cpm-policy", policy),
        ("content-type", content_type),
    ] {
        if let Some(value) = value {
            builder = builder.header(name, value);
        }
    }
    builder.body(Body::from("{}")).expect("probe request")
}
async fn assert_error(
    response: Response,
    status: StatusCode,
    code: &str,
    correlation: &str,
    observation: &str,
) {
    assert_eq!(response.status(), status);
    assert_eq!(
        response.headers()["x-correlation-id"].to_str().unwrap(),
        correlation
    );
    assert_eq!(
        response.headers()["x-cpm-middleware-order"]
            .to_str()
            .unwrap(),
        MIDDLEWARE_ORDER
    );
    assert_eq!(
        response.headers()["x-cpm-observation"].to_str().unwrap(),
        observation
    );
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let envelope: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope["code"], code);
    assert_eq!(envelope["correlation_id"], correlation);
}
#[tokio::test]
async fn middleware_matrix_denials_and_validation_are_effect_free() {
    let probe = EffectProbe::new(Duration::ZERO);
    let cases = [
        (
            None,
            Some("reviewer"),
            None,
            Some("application/json"),
            StatusCode::UNAUTHORIZED,
            "authn_required",
            "authn_denied",
        ),
        (
            Some("Bearer "),
            Some("reviewer"),
            None,
            Some("application/json"),
            StatusCode::UNAUTHORIZED,
            "authn_required",
            "authn_denied",
        ),
        (
            Some("Bearer token"),
            Some("observer"),
            None,
            Some("application/json"),
            StatusCode::FORBIDDEN,
            "authz_denied",
            "authz_denied",
        ),
        (
            Some("Bearer token"),
            Some("reviewer"),
            Some("deny"),
            Some("application/json"),
            StatusCode::FORBIDDEN,
            "policy_denied",
            "policy_denied",
        ),
        (
            Some("Bearer token"),
            Some("reviewer"),
            None,
            Some("text/plain"),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "validation_failed",
            "validation_denied",
        ),
    ];
    for (index, (auth, role, policy, content_type, status, code, observation)) in
        cases.into_iter().enumerate()
    {
        let correlation = format!("matrix-denial-{index}");
        let response = probe_router(probe.clone(), Duration::from_secs(1))
            .oneshot(request(auth, role, policy, content_type, &correlation))
            .await
            .unwrap();
        assert_error(response, status, code, &correlation, observation).await;
        probe.assert_untouched();
    }
}
#[tokio::test]
async fn middleware_matrix_success_exposes_correlation_order_and_observation() {
    let probe = EffectProbe::new(Duration::ZERO);
    let correlation = "matrix-success";
    let response = probe_router(probe.clone(), Duration::from_secs(1))
        .oneshot(request(
            Some("Bearer token"),
            Some("admin"),
            None,
            Some("application/json; charset=utf-8"),
            correlation,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-correlation-id"], correlation);
    assert_eq!(
        response.headers()["x-cpm-middleware-order"],
        MIDDLEWARE_ORDER
    );
    assert_eq!(response.headers()["x-cpm-observation"], "completed");
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert!(probe.marker.exists());
}
#[tokio::test]
async fn middleware_matrix_timeout_cancels_before_upstream_and_durable_effects() {
    let probe = EffectProbe::new(Duration::from_millis(80));
    let correlation = "matrix-timeout";
    let response = probe_router(probe.clone(), Duration::from_millis(5))
        .oneshot(request(
            Some("Bearer token"),
            Some("reviewer"),
            None,
            Some("application/json"),
            correlation,
        ))
        .await
        .unwrap();
    assert_error(
        response,
        StatusCode::REQUEST_TIMEOUT,
        "timeout",
        correlation,
        "timeout",
    )
    .await;
    sleep(Duration::from_millis(100)).await;
    probe.assert_untouched();
}
#[tokio::test]
async fn middleware_matrix_maps_application_errors_to_typed_correlated_envelopes() {
    let data = std::env::temp_dir().join(format!("cpm-matrix-store-{}", Uuid::new_v4()));
    let state = AppState::open(&data, FsPath::new("/missing")).unwrap();
    let correlation = "matrix-typed-error";
    let response = router(state)
        .oneshot(
            HttpRequest::builder()
                .uri("/api/v1/registry/not-stored/1.0.0")
                .header("authorization", "Bearer token")
                .header("x-cpm-role", "reviewer")
                .header("x-correlation-id", correlation)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_error(
        response,
        StatusCode::NOT_FOUND,
        "application_error",
        correlation,
        "completed",
    )
    .await;
}
#[tokio::test]
async fn middleware_matrix_replaces_untrusted_correlation_with_bounded_uuid() {
    let probe = EffectProbe::new(Duration::ZERO);
    let supplied = "x".repeat(97);
    let response = probe_router(probe.clone(), Duration::from_secs(1))
        .oneshot(request(
            None,
            Some("reviewer"),
            None,
            Some("application/json"),
            &supplied,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let correlation = response.headers()["x-correlation-id"]
        .to_str()
        .unwrap()
        .to_owned();
    assert_ne!(correlation, supplied);
    Uuid::parse_str(&correlation).expect("generated correlation is a UUID");
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let envelope: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope["correlation_id"], correlation);
    probe.assert_untouched();
}
