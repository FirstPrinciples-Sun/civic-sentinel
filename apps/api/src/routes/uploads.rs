use std::path::{Path, PathBuf};

use axum::{
    extract::Multipart,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

const MAX_IMAGE_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB

fn file_extension_from_name_or_type(file_name: Option<&str>, content_type: &str) -> &'static str {
    if let Some(name) = file_name {
        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
            return match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => "jpg",
                "png" => "png",
                "webp" => "webp",
                "gif" => "gif",
                "bmp" => "bmp",
                _ => "bin",
            };
        }
    }

    match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        _ => "bin",
    }
}

fn upload_dir() -> PathBuf {
    std::env::var("UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("uploads"))
}

pub async fn upload_issue_media(mut multipart: Multipart) -> (StatusCode, Json<Value>) {
    let base_dir = upload_dir();
    if let Err(e) = tokio::fs::create_dir_all(&base_dir).await {
        tracing::error!("Failed to create upload directory: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": "Failed to prepare upload storage"
            })),
        );
    }

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(e) => {
                tracing::error!("Failed to read multipart field: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid multipart request"
                    })),
                );
            }
        };

        if field.name() != Some("file") {
            continue;
        }

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        if !content_type.starts_with("image/") {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Only image uploads are supported"
                })),
            );
        }

        let original_name = field.file_name().map(str::to_string);
        let ext = file_extension_from_name_or_type(original_name.as_deref(), &content_type);

        let bytes = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to read multipart bytes: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Invalid upload payload"
                    })),
                );
            }
        };

        if bytes.len() > MAX_IMAGE_SIZE_BYTES {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(json!({
                    "success": false,
                    "error": "Image exceeds max size of 10MB"
                })),
            );
        }

        let file_name = format!("{}-{}.{}", chrono::Utc::now().timestamp(), Uuid::new_v4(), ext);
        let file_path = base_dir.join(&file_name);

        if let Err(e) = tokio::fs::write(&file_path, &bytes).await {
            tracing::error!("Failed to save uploaded file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to store uploaded file"
                })),
            );
        }

        return (
            StatusCode::CREATED,
            Json(json!({
                "success": true,
                "data": {
                    "url": format!("/uploads/{}", file_name),
                    "file_name": original_name.unwrap_or(file_name),
                    "size_bytes": bytes.len(),
                    "content_type": content_type
                },
                "message": "File uploaded successfully"
            })),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "No file field found. Use multipart field name 'file'"
        })),
    )
}
