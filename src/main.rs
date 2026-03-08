use askama::Template;
use axum::{
    body::Body,
    extract::{Path, Request, State},
    middleware,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
    Router,
};
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tower_http::services::ServeDir;

// ── Provider config ──────────────────────────────────────────────────────────

const PROVIDERS: &[(&str, &str)] = &[
    ("anthropic", "https://api.anthropic.com"),
    ("openai", "https://api.openai.com"),
    ("gemini", "https://generativelanguage.googleapis.com"),
];

fn provider_target(name: &str) -> Option<&'static str> {
    PROVIDERS.iter().find(|(n, _)| *n == name).map(|(_, target)| *target)
}

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ApiKeys {
    anthropic: Option<String>,
    openai: Option<String>,
    gemini: Option<String>,
}

impl ApiKeys {
    fn get(&self, provider: &str) -> Option<&str> {
        match provider {
            "anthropic" => self.anthropic.as_deref(),
            "openai" => self.openai.as_deref(),
            "gemini" => self.gemini.as_deref(),
            _ => None,
        }
    }

    fn has_any(&self) -> bool {
        self.anthropic.is_some() || self.openai.is_some() || self.gemini.is_some()
    }
}

struct RateBucket {
    count: u32,
    window_start: Instant,
}

#[derive(Clone)]
struct AppState {
    proxy_url: String,
    http: reqwest::Client,
    api_keys: ApiKeys,
    rate: Arc<DashMap<IpAddr, RateBucket>>,
    daily_requests: Arc<AtomicU64>,
    day_start: Arc<std::sync::Mutex<Instant>>,
    hourly_requests: Arc<AtomicU64>,
    hour_start: Arc<std::sync::Mutex<Instant>>,
}

const RATE_LIMIT_PER_HOUR: u32 = 5_000;  // per IP
const DAILY_REQUEST_CAP: u64 = 50_000;  // ~$1500 at ~$0.03/req
const HOURLY_SOFT_CAP: u64 = 3_000;     // ~$100/hr → switch to cheap models

// ── Model randomization ──────────────────────────────────────────────────────

const EXPENSIVE_MODELS: &[&str] = &[
    "claude-opus-4-6",
    "gpt-5.2",
    "gemini-3-pro-preview",
];

const CHEAP_MODELS: &[&str] = &[
    "claude-haiku-4-5",
    "gpt-5-mini",
    "gemini-3.1-flash-lite",
];

/// Pick n models, tier based on current global request rate.
/// Under soft cap → expensive models. Over → cheap.
fn pick_models(state: &AppState, n: usize) -> Vec<&'static str> {
    let hourly = current_hourly_requests(state);
    let tier = if hourly < HOURLY_SOFT_CAP {
        EXPENSIVE_MODELS
    } else {
        CHEAP_MODELS
    };
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;
    // Shuffle within the tier so each faction gets a different provider
    let mut indices: Vec<usize> = (0..tier.len()).collect();
    let mut seed = t;
    for i in (1..indices.len()).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = seed % (i + 1);
        indices.swap(i, j);
    }
    // Cycle through shuffled tier to fill n slots
    indices.iter().cycle().take(n).map(|&i| tier[i]).collect()
}

fn current_hourly_requests(state: &AppState) -> u64 {
    let now = Instant::now();
    let start = *state.hour_start.lock().unwrap();
    if now.duration_since(start).as_secs() > 3600 {
        0 // window expired, will reset on next request
    } else {
        state.hourly_requests.load(Ordering::Relaxed)
    }
}

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage;

#[derive(Template)]
#[template(path = "demo_diplomacy.html")]
struct DemoDiplomacyPage {
    proxy_url: String,
    france: String,
    prussia: String,
    russia: String,
    narrator: String,
}

#[derive(Template)]
#[template(path = "demo_webcm.html")]
struct DemoWebcmPage {
    proxy_url: String,
    orchestrator: String,
    planner: String,
    coder: String,
    reviewer: String,
}

#[derive(Template)]
#[template(path = "demo_office.html")]
struct DemoOfficePage {
    proxy_url: String,
    model: String,
}

async fn home() -> Html<String> {
    Html(HomePage.render().unwrap())
}

async fn demo_diplomacy(State(state): State<AppState>) -> Html<String> {
    let m = pick_models(&state, 4);
    Html(DemoDiplomacyPage {
        proxy_url: state.proxy_url.clone(),
        france: m[0].to_string(),
        prussia: m[1].to_string(),
        russia: m[2].to_string(),
        narrator: m[3].to_string(),
    }.render().unwrap())
}

async fn demo_webcm(State(state): State<AppState>) -> Html<String> {
    let m = pick_models(&state, 4);
    Html(DemoWebcmPage {
        proxy_url: state.proxy_url.clone(),
        orchestrator: m[0].to_string(),
        planner: m[1].to_string(),
        coder: m[2].to_string(),
        reviewer: m[3].to_string(),
    }.render().unwrap())
}

async fn demo_office(State(state): State<AppState>) -> Html<String> {
    let hourly = current_hourly_requests(&state);
    let model = if hourly < HOURLY_SOFT_CAP { "claude-opus-4-6" } else { "claude-haiku-4-5" };
    Html(DemoOfficePage {
        proxy_url: state.proxy_url.clone(),
        model: model.to_string(),
    }.render().unwrap())
}

// ── CORS ─────────────────────────────────────────────────────────────────────

fn cors_headers(resp: &mut Response) {
    let h = resp.headers_mut();
    h.insert("access-control-allow-origin", "*".parse().unwrap());
    h.insert("access-control-allow-methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    h.insert("access-control-allow-headers",
        "content-type, authorization, x-api-key, x-goog-api-key, anthropic-version, anthropic-beta, anthropic-dangerous-direct-browser-access"
            .parse().unwrap());
    h.insert("access-control-expose-headers", "*".parse().unwrap());
    h.insert("access-control-max-age", "86400".parse().unwrap());
}

async fn cors_preflight() -> Response {
    let mut resp = Response::builder()
        .status(204)
        .body(Body::empty())
        .unwrap();
    cors_headers(&mut resp);
    resp
}

// ── Rate limiting ────────────────────────────────────────────────────────────

fn client_ip(req: &Request) -> IpAddr {
    // Cloud Run sets x-forwarded-for; use first IP in chain
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(s) = xff.to_str() {
            if let Some(first) = s.split(',').next() {
                if let Ok(ip) = first.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }
    // Fallback for local dev
    "127.0.0.1".parse().unwrap()
}

fn check_rate_limit(state: &AppState, ip: IpAddr) -> Result<(), Response> {
    let now = Instant::now();

    // Global hourly counter (for tier switching)
    {
        let mut hour = state.hour_start.lock().unwrap();
        if now.duration_since(*hour).as_secs() > 3600 {
            state.hourly_requests.store(0, Ordering::Relaxed);
            *hour = now;
        }
    }
    state.hourly_requests.fetch_add(1, Ordering::Relaxed);

    // Daily global cap
    {
        let mut day = state.day_start.lock().unwrap();
        if now.duration_since(*day).as_secs() > 86400 {
            state.daily_requests.store(0, Ordering::Relaxed);
            *day = now;
        }
    }
    let daily = state.daily_requests.fetch_add(1, Ordering::Relaxed);
    if daily >= DAILY_REQUEST_CAP {
        let mut resp = Response::builder()
            .status(503)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"error":"daily_cap","message":"Demo daily limit reached. Try again tomorrow."}"#))
            .unwrap();
        cors_headers(&mut resp);
        return Err(resp);
    }

    // Per-IP hourly limit
    let mut entry = state.rate.entry(ip).or_insert_with(|| RateBucket {
        count: 0,
        window_start: now,
    });
    let bucket = entry.value_mut();
    if now.duration_since(bucket.window_start).as_secs() > 3600 {
        bucket.count = 0;
        bucket.window_start = now;
    }
    bucket.count += 1;
    if bucket.count > RATE_LIMIT_PER_HOUR {
        let mut resp = Response::builder()
            .status(429)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"error":"rate_limited","message":"Too many requests. Limit: 500/hour."}"#))
            .unwrap();
        cors_headers(&mut resp);
        return Err(resp);
    }

    Ok(())
}

// ── LLM Proxy handler ───────────────────────────────────────────────────────

const STRIP_REQUEST_HEADERS: &[&str] = &[
    "host", "connection", "keep-alive", "transfer-encoding",
    "accept-encoding", "origin", "referer",
];

const STRIP_RESPONSE_HEADERS: &[&str] = &[
    "content-encoding", "transfer-encoding",
];

async fn llm_proxy_root(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    req: Request,
) -> Response {
    llm_proxy_inner(state, &provider, "/", req).await
}

async fn llm_proxy(
    State(state): State<AppState>,
    Path((provider, path)): Path<(String, String)>,
    req: Request,
) -> Response {
    llm_proxy_inner(state, &provider, &path, req).await
}

async fn llm_proxy_inner(
    state: AppState,
    provider: &str,
    path: &str,
    req: Request,
) -> Response {
    // CORS preflight
    if req.method() == axum::http::Method::OPTIONS {
        return cors_preflight().await;
    }

    // Check provider exists and has a key
    let target = match provider_target(provider) {
        Some(t) => t,
        None => {
            let mut resp = Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"error":"unknown_provider"}"#))
                .unwrap();
            cors_headers(&mut resp);
            return resp;
        }
    };
    let api_key = match state.api_keys.get(provider) {
        Some(k) => k.to_string(),
        None => {
            let mut resp = Response::builder()
                .status(503)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"error":"provider_not_configured"}"#))
                .unwrap();
            cors_headers(&mut resp);
            return resp;
        }
    };

    // Rate limit
    let ip = client_ip(&req);
    if let Err(resp) = check_rate_limit(&state, ip) {
        let count = state.hourly_requests.load(Ordering::Relaxed);
        eprintln!("RATE LIMITED: ip={ip} hourly={count} provider={provider} path={path}");
        return resp;
    }
    let count = state.hourly_requests.load(Ordering::Relaxed);
    if count % 50 == 0 {
        eprintln!("Proxy request #{count}: {provider}/{path}");
    }

    // Build target URL
    let path = path.trim_start_matches('/');
    let mut target_url = format!("{}/{}", target, path);
    if let Some(query) = req.uri().query() {
        target_url.push('?');
        target_url.push_str(query);
    }

    // Gemini: strip ?key= query param (legacy client-side auth)
    if provider == "gemini" {
        if let Ok(mut url) = reqwest::Url::parse(&target_url) {
            let pairs: Vec<(String, String)> = url.query_pairs()
                .filter(|(k, _)| k != "key")
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
            url.query_pairs_mut().clear().extend_pairs(&pairs);
            if url.query() == Some("") {
                url.set_query(None);
            }
            target_url = url.to_string();
        }
    }

    // Build upstream request headers
    let method = req.method().clone();
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in req.headers() {
        let name_lower = name.as_str().to_lowercase();
        if STRIP_REQUEST_HEADERS.contains(&name_lower.as_str()) {
            continue;
        }
        if let Ok(n) = reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()) {
            if let Ok(v) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                headers.insert(n, v);
            }
        }
    }

    // Inject auth per provider
    match provider {
        "anthropic" => {
            headers.insert("x-api-key", api_key.parse().unwrap());
            headers.remove("anthropic-dangerous-direct-browser-access");
        }
        "openai" => {
            headers.insert("authorization", format!("Bearer {api_key}").parse().unwrap());
        }
        "gemini" => {
            headers.insert("x-goog-api-key", api_key.parse().unwrap());
        }
        _ => {}
    }

    // Forward request
    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            let mut resp = Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"error":"bad_request","message":"{}"}}"#, e)))
                .unwrap();
            cors_headers(&mut resp);
            return resp;
        }
    };

    let upstream_req = state.http
        .request(method, &target_url)
        .headers(headers)
        .body(body_bytes);

    let upstream = match upstream_req.send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Proxy error: {e}");
            let mut resp = Response::builder()
                .status(502)
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"error":"proxy_error","message":"{}"}}"#, e)))
                .unwrap();
            cors_headers(&mut resp);
            return resp;
        }
    };

    // Build response
    let status = upstream.status();
    let mut resp_headers = axum::http::HeaderMap::new();
    for (name, value) in upstream.headers() {
        let name_lower = name.as_str().to_lowercase();
        if STRIP_RESPONSE_HEADERS.contains(&name_lower.as_str()) {
            continue;
        }
        resp_headers.insert(name.clone(), value.clone());
    }

    // Stream response body
    let stream = upstream.bytes_stream();
    let body = Body::from_stream(stream);

    let mut resp = Response::builder()
        .status(status.as_u16())
        .body(body)
        .unwrap();
    *resp.headers_mut() = resp_headers;
    cors_headers(&mut resp);
    resp
}

// ── WASM-friendly response headers ───────────────────────────────────────────

async fn wasm_headers(req: axum::http::Request<Body>, next: Next) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();

    if path.ends_with(".wasm") {
        headers.insert("content-type", "application/wasm".parse().unwrap());
    }

    // Hashed assets (Vite content-hashed filenames) get long cache
    if path.starts_with("/demos/") && path.contains("/assets/") {
        headers.insert("cache-control", "public, max-age=86400, immutable".parse().unwrap());
    }

    // Unhashed demo files (runtime.js, .wasm) — never use stale cache
    // But allow caching for images (sprites, backgrounds)
    if path.starts_with("/demos/") && !path.contains("/assets/")
        && !path.ends_with(".png") && !path.ends_with(".jpg") && !path.ends_with(".svg")
    {
        headers.insert("cache-control", "no-store".parse().unwrap());
    }

    // WebCM and Office need cross-origin isolation for SharedArrayBuffer
    if path.starts_with("/demos/webcm/") || path.starts_with("/demos/office/") {
        headers.insert("cross-origin-opener-policy", "same-origin".parse().unwrap());
        headers.insert("cross-origin-embedder-policy", "require-corp".parse().unwrap());
    }

    resp
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let api_keys = ApiKeys {
        anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
        openai: std::env::var("OPENAI_API_KEY").ok(),
        gemini: std::env::var("GEMINI_API_KEY").ok(),
    };

    let proxy_url = std::env::var("PROXY_URL").unwrap_or_else(|_| {
        if api_keys.has_any() {
            format!("http://localhost:{port}/api/llm")
        } else {
            String::new()
        }
    });

    if api_keys.has_any() {
        let mut providers = Vec::new();
        if api_keys.anthropic.is_some() { providers.push("anthropic"); }
        if api_keys.openai.is_some() { providers.push("openai"); }
        if api_keys.gemini.is_some() { providers.push("gemini"); }
        println!("LLM proxy enabled: {}", providers.join(", "));
    }

    let state = AppState {
        proxy_url,
        http: reqwest::Client::new(),
        api_keys,
        rate: Arc::new(DashMap::new()),
        daily_requests: Arc::new(AtomicU64::new(0)),
        day_start: Arc::new(std::sync::Mutex::new(Instant::now())),
        hourly_requests: Arc::new(AtomicU64::new(0)),
        hour_start: Arc::new(std::sync::Mutex::new(Instant::now())),
    };

    let app = Router::new()
        .route("/", get(home))
        .route("/demos/diplomacy", get(demo_diplomacy))
        .route("/demos/diplomacy/", get(demo_diplomacy))
        .route("/demos/webcm", get(demo_webcm))
        .route("/demos/webcm/", get(demo_webcm))
        .route("/demos/office", get(demo_office))
        .route("/demos/office/", get(demo_office))
        // LLM proxy (handler checks for OPTIONS internally)
        .route("/api/llm/{provider}/{*path}", any(llm_proxy))
        .route("/api/llm/{provider}", any(llm_proxy_root))
        // Static files
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/demos/diplomacy/app",
            ServeDir::new("demos/diplomacy/app").precompressed_gzip())
        .nest_service("/demos/webcm/app",
            ServeDir::new("demos/webcm/app").precompressed_gzip())
        .nest_service("/demos/office/app",
            ServeDir::new("demos/office/app").precompressed_gzip())
        .layer(middleware::from_fn(wasm_headers))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://localhost:{port}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
