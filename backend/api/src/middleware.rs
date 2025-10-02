use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use crate::{
    models::AppState,
    error::{ApiError, Result},
    auth::validate_jwt,
    cache::CacheService,
};

// Authentication middleware
pub async fn auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    // Skip auth for public endpoints
    let path = request.uri().path();
    if is_public_endpoint(path) {
        return Ok(next.run(request).await);
    }

    // Extract Bearer token
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(&auth_str[7..])
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::Authentication("Missing or invalid authorization header".to_string()))?;

    // Validate JWT token
    let claims = validate_jwt(auth_header, &state.config.jwt_secret)
        .map_err(|e| ApiError::Authentication(format!("Invalid token: {}", e)))?;

    // Add user info to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Rate limiting middleware
pub async fn rate_limit(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response> {
    // Get client identifier (IP or user ID if authenticated)
    let client_id = get_client_identifier(&headers, &request);
    
    // Create cache service
    let mut cache = CacheService::new(state.redis.clone());
    
    // Check rate limit
    let rate_limit_info = cache.check_rate_limit(
        &client_id,
        state.config.rate_limit.requests_per_minute as u32,
        Duration::from_secs(60),
    ).await?;
    
    if rate_limit_info.requests_remaining == 0 {
        return Err(ApiError::RateLimit);
    }
    
    // Add rate limit headers to response
    let mut response = next.run(request).await;
    
    let headers_mut = response.headers_mut();
    headers_mut.insert(
        "X-RateLimit-Limit",
        rate_limit_info.window_size.to_string().parse().unwrap(),
    );
    headers_mut.insert(
        "X-RateLimit-Remaining",
        rate_limit_info.requests_remaining.to_string().parse().unwrap(),
    );
    headers_mut.insert(
        "X-RateLimit-Reset",
        rate_limit_info.reset_time.timestamp().to_string().parse().unwrap(),
    );
    
    Ok(response)
}

// CORS middleware (handled by tower-http, but we can add custom logic here)
pub async fn cors(
    request: Request,
    next: Next,
) -> Result<Response> {
    let response = next.run(request).await;
    // Additional CORS logic can be added here if needed
    Ok(response)
}

// Request logging middleware
pub async fn request_logging(
    request: Request,
    next: Next,
) -> Result<Response> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = duration.as_millis(),
        "Request completed"
    );
    
    Ok(response)
}

// Security headers middleware
pub async fn security_headers(
    request: Request,
    next: Next,
) -> Result<Response> {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap(),
    );
    headers.insert(
        "X-Frame-Options",
        "DENY".parse().unwrap(),
    );
    headers.insert(
        "X-XSS-Protection",
        "1; mode=block".parse().unwrap(),
    );
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    
    Ok(response)
}

// Helper functions
fn is_public_endpoint(path: &str) -> bool {
    matches!(path,
        "/health" |
        "/health/ready" |
        "/health/live" |
        "/metrics" |
        "/api/v1/analytics/public" |
        "/api/v1/chains" |
        "/api/v1/tokens/prices" |
        "/ws" | // WebSocket endpoint (auth handled separately)
        "/api/v1/intents" if path.ends_with("/status") // Public intent status
    )
}

fn get_client_identifier(headers: &HeaderMap, request: &Request) -> String {
    // Try to get user ID from JWT claims in extensions
    if let Some(claims) = request.extensions().get::<crate::models::Claims>() {
        return format!("user:{}", claims.sub);
    }
    
    // Fall back to IP address
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return format!("ip:{}", ip.trim());
            }
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return format!("ip:{}", ip_str);
        }
    }
    
    // Fall back to connection info (this might not be available in all setups)
    if let Some(connect_info) = request.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>() {
        return format!("ip:{}", connect_info.ip());
    }
    
    // Default fallback
    "unknown".to_string()
}

// Metrics middleware
pub async fn metrics(
    request: Request,
    next: Next,
) -> Result<Response> {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status_code = response.status().as_u16();
    
    // Record metrics
    metrics::counter!("http_requests_total", "method" => method.to_string(), "path" => uri.clone(), "status" => status_code.to_string()).increment(1);
    metrics::histogram!("http_request_duration_seconds", "method" => method.to_string(), "path" => uri).record(duration.as_secs_f64());
    
    Ok(response)
}

// Content validation middleware
pub async fn content_validation(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response> {
    // Validate content-type for POST/PUT requests
    if matches!(request.method(), &axum::http::Method::POST | &axum::http::Method::PUT) {
        if let Some(content_type) = headers.get("content-type") {
            let content_type_str = content_type.to_str()
                .map_err(|_| ApiError::BadRequest("Invalid content-type header".to_string()))?;
            
            if !content_type_str.starts_with("application/json") {
                return Err(ApiError::BadRequest(
                    "Content-Type must be application/json".to_string()
                ));
            }
        } else {
            return Err(ApiError::BadRequest(
                "Content-Type header is required".to_string()
            ));
        }
    }
    
    Ok(next.run(request).await)
}

// Request size limit middleware
pub async fn request_size_limit(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response> {
    const MAX_CONTENT_LENGTH: u64 = 1024 * 1024; // 1MB
    
    if let Some(content_length) = headers.get("content-length") {
        let length: u64 = content_length.to_str()
            .map_err(|_| ApiError::BadRequest("Invalid content-length header".to_string()))?
            .parse()
            .map_err(|_| ApiError::BadRequest("Invalid content-length value".to_string()))?;
        
        if length > MAX_CONTENT_LENGTH {
            return Err(ApiError::BadRequest(
                format!("Request body too large. Maximum size is {} bytes", MAX_CONTENT_LENGTH)
            ));
        }
    }
    
    Ok(next.run(request).await)
}

// API versioning middleware
pub async fn api_versioning(
    request: Request,
    next: Next,
) -> Result<Response> {
    // Extract API version from path or headers
    let path = request.uri().path();
    
    // Ensure API version is supported
    if path.starts_with("/api/") {
        if !path.starts_with("/api/v1/") {
            return Err(ApiError::BadRequest(
                "Unsupported API version. Please use /api/v1/".to_string()
            ));
        }
    }
    
    Ok(next.run(request).await)
}

// Error handling middleware (catches panics and converts to proper responses)
pub async fn error_handling(
    request: Request,
    next: Next,
) -> Response {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Handle::current().block_on(async {
            next.run(request).await
        })
    })) {
        Ok(response) => response,
        Err(_) => {
            tracing::error!("Request handler panicked");
            ApiError::Internal("Internal server error".to_string()).into_response()
        }
    }
}