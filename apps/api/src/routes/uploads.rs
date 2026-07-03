use std::path::{Path, PathBuf};

use axum::{
    extract::Multipart,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

// Security: Maximum file size to prevent DoS attacks via large uploads
const MAX_IMAGE_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB

// Security: Whitelist of allowed MIME types
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
];

// Security: Whitelist of allowed file extensions
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

/// Sanitize filename to prevent path traversal attacks
/// Removes: .. / \ : * ? " < > |
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| {
            c.is_alphanumeric()
            || *c == '.'
            || *c == '-'
            || *c == '_'
        })
        .take(100) // Limit filename length
        .collect()
}

/// Validate file extension against whitelist
fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

/// Validate MIME type against whitelist
fn is_allowed_mime_type(mime: &str) -> bool {
    ALLOWED_MIME_TYPES.contains(&mime)
}

/// Extract and validate file extension from filename or content type
fn file_extension_from_name_or_type(file_name: Option<&str>, content_type: &str) -> Option<&'static str> {
    // Try to extract from filename first
    if let Some(name) = file_name {
        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if is_allowed_extension(&ext_lower) {
                return match ext_lower.as_str() {
                    "jpg" | "jpeg" => Some("jpg"),
                    "png" => Some("png"),
                    "webp" => Some("webp"),
                    "gif" => Some("gif"),
                    _ => None,
                };
            }
        }
    }

    // Fallback to MIME type
    if is_allowed_mime_type(content_type) {
        return match content_type {
            "image/jpeg" => Some("jpg"),
            "image/png" => Some("png"),
            "image/webp" => Some("webp"),
            "image/gif" => Some("gif"),
            _ => None,
        };
    }

    None
}

/// Verify image file content by checking magic bytes (file signature)
/// This prevents uploading malicious files with fake extensions
fn verify_image_magic_bytes(bytes: &[u8], expected_type: &str) -> bool {
    if bytes.len() < 12 {
        return false;
    }

    match expected_type {
        "jpg" => {
            // JPEG: FF D8 FF
            bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF
        }
        "png" => {
            // PNG: 89 50 4E 47 0D 0A 1A 0A
            bytes.len() >= 8
                && bytes[0] == 0x89
                && bytes[1] == 0x50
                && bytes[2] == 0x4E
                && bytes[3] == 0x47
                && bytes[4] == 0x0D
                && bytes[5] == 0x0A
                && bytes[6] == 0x1A
                && bytes[7] == 0x0A
        }
        "gif" => {
            // GIF: 47 49 46 38 (GIF8)
            bytes.len() >= 4
                && bytes[0] == 0x47
                && bytes[1] == 0x49
                && bytes[2] == 0x46
                && bytes[3] == 0x38
        }
        "webp" => {
            // WebP: RIFF ... WEBP
            bytes.len() >= 12
                && bytes[0] == 0x52 // R
                && bytes[1] == 0x49 // I
                && bytes[2] == 0x46 // F
                && bytes[3] == 0x46 // F
                && bytes[8] == 0x57 // W
                && bytes[9] == 0x45 // E
                && bytes[10] == 0x42 // B
                && bytes[11] == 0x50 // P
        }
        _ => false,
    }
}
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

        // Validate Content-Type header
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !is_allowed_mime_type(&content_type) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid file type. Only JPEG, PNG, WebP, and GIF images are allowed"
                })),
            );
        }

        // Sanitize original filename to prevent path traversal
        let original_name = field.file_name().map(|name| sanitize_filename(name));

        // Determine and validate file extension
        let ext = match file_extension_from_name_or_type(original_name.as_deref(), &content_type) {
            Some(e) => e,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "Unsupported file extension or MIME type"
                    })),
                );
            }
        };

        // Read file bytes
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

        // Validate file size
        if bytes.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Empty file not allowed"
                })),
            );
        }

        if bytes.len() > MAX_IMAGE_SIZE_BYTES {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(json!({
                    "success": false,
                    "error": format!("Image exceeds maximum size of {} MB", MAX_IMAGE_SIZE_BYTES / (1024 * 1024))
                })),
            );
        }

        // Verify file content matches declared type (magic bytes check)
        if !verify_image_magic_bytes(&bytes, ext) {
            tracing::warn!(
                "File magic bytes don't match declared type. Content-Type: {}, Extension: {}",
                content_type,
                ext
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "File content does not match declared type. Possible file corruption or spoofing attempt."
                })),
            );
        }

        // Generate secure filename: timestamp-uuid.ext
        let file_name = format!("{}-{}.{}", chrono::Utc::now().timestamp(), Uuid::new_v4(), ext);
        let file_path = base_dir.join(&file_name);

        // Write file to disk
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

        tracing::info!(
            "File uploaded successfully: {} ({} bytes, type: {})",
            file_name,
            bytes.len(),
            content_type
        );

        return (
            StatusCode::CREATED,
            Json(json!({
                "success": true,
                "data": {
                    "url": format!("/uploads/{}", file_name),
                    "file_name": original_name.unwrap_or_else(|| file_name.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.jpg"), "test.jpg");
        assert_eq!(sanitize_filename("my-photo_2023.png"), "my-photo_2023.png");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("test<script>.jpg"), "testscript.jpg");
        assert_eq!(sanitize_filename("file:name.png"), "filename.png");

        // Test length limit
        let long_name = "a".repeat(200);
        assert_eq!(sanitize_filename(&long_name).len(), 100);
    }

    #[test]
    fn test_is_allowed_extension() {
        assert!(is_allowed_extension("jpg"));
        assert!(is_allowed_extension("jpeg"));
        assert!(is_allowed_extension("png"));
        assert!(is_allowed_extension("webp"));
        assert!(is_allowed_extension("gif"));
        assert!(is_allowed_extension("JPG")); // Case insensitive

        assert!(!is_allowed_extension("exe"));
        assert!(!is_allowed_extension("sh"));
        assert!(!is_allowed_extension("php"));
        assert!(!is_allowed_extension("svg")); // SVG can contain scripts
    }

    #[test]
    fn test_is_allowed_mime_type() {
        assert!(is_allowed_mime_type("image/jpeg"));
        assert!(is_allowed_mime_type("image/png"));
        assert!(is_allowed_mime_type("image/webp"));
        assert!(is_allowed_mime_type("image/gif"));

        assert!(!is_allowed_mime_type("image/svg+xml")); // Can contain scripts
        assert!(!is_allowed_mime_type("application/octet-stream"));
        assert!(!is_allowed_mime_type("text/html"));
    }

    #[test]
    fn test_verify_image_magic_bytes_jpeg() {
        // Valid JPEG
        let jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert!(verify_image_magic_bytes(&jpeg_bytes, "jpg"));

        // Invalid JPEG
        let fake_jpeg = vec![0x00, 0x00, 0x00, 0x00];
        assert!(!verify_image_magic_bytes(&fake_jpeg, "jpg"));
    }

    #[test]
    fn test_verify_image_magic_bytes_png() {
        // Valid PNG
        let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(verify_image_magic_bytes(&png_bytes, "png"));

        // Invalid PNG
        let fake_png = vec![0x00, 0x00, 0x00, 0x00];
        assert!(!verify_image_magic_bytes(&fake_png, "png"));
    }

    #[test]
    fn test_verify_image_magic_bytes_gif() {
        // Valid GIF
        let gif_bytes = vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61]; // GIF89a
        assert!(verify_image_magic_bytes(&gif_bytes, "gif"));

        // Invalid GIF
        let fake_gif = vec![0x00, 0x00, 0x00, 0x00];
        assert!(!verify_image_magic_bytes(&fake_gif, "gif"));
    }

    #[test]
    fn test_verify_image_magic_bytes_webp() {
        // Valid WebP
        let webp_bytes = vec![
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x00, 0x00, 0x00, 0x00, // size (doesn't matter for this test)
            0x57, 0x45, 0x42, 0x50, // WEBP
        ];
        assert!(verify_image_magic_bytes(&webp_bytes, "webp"));

        // Invalid WebP
        let fake_webp = vec![0x00; 12];
        assert!(!verify_image_magic_bytes(&fake_webp, "webp"));
    }

    #[test]
    fn test_file_extension_validation() {
        // Valid combinations
        assert_eq!(
            file_extension_from_name_or_type(Some("photo.jpg"), "image/jpeg"),
            Some("jpg")
        );
        assert_eq!(
            file_extension_from_name_or_type(Some("photo.png"), "image/png"),
            Some("png")
        );

        // Invalid extension with valid MIME
        assert_eq!(
            file_extension_from_name_or_type(Some("photo.exe"), "image/jpeg"),
            Some("jpg") // Falls back to MIME type
        );

        // Invalid MIME type
        assert_eq!(
            file_extension_from_name_or_type(Some("photo.jpg"), "application/octet-stream"),
            None
        );

        // No filename, use MIME
        assert_eq!(
            file_extension_from_name_or_type(None, "image/png"),
            Some("png")
        );
    }
}
