use base64::Engine;
use twilight_model::id::Id;

/// Parse a string snowflake ID into a typed twilight Id<T>.
pub fn parse_id<T>(s: &str) -> Result<Id<T>, rmcp::ErrorData> {
    let raw: u64 = s.trim().parse().map_err(|_| {
        rmcp::ErrorData::invalid_params(
            format!("Invalid ID '{s}': must be a numeric snowflake ID"),
            None,
        )
    })?;
    if raw == 0 {
        return Err(rmcp::ErrorData::invalid_params(
            "ID must be non-zero",
            None,
        ));
    }
    Ok(Id::new(raw))
}

/// Read a file and return it as a data URI (data:mime;base64,...).
/// Supports image formats (png, jpg, gif, webp) and audio formats (mp3, ogg).
pub fn read_file_as_data_uri(path: &str) -> Result<String, rmcp::ErrorData> {
    let mime = match path.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        _ => {
            return Err(rmcp::ErrorData::invalid_params(
                "Unsupported file format. Supported: png, jpg, gif, webp, mp3, ogg.",
                None,
            ))
        }
    };

    let bytes = std::fs::read(path).map_err(|e| {
        rmcp::ErrorData::invalid_params(format!("Failed to read file: {e}"), None)
    })?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
