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
