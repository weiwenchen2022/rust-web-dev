use handle_errors::Error;
use std::collections::HashMap;

/// Pagination struct that is getting extracted
/// from query params
#[derive(Debug, Default, PartialEq)]
pub struct Pagination {
    /// The index of the first item that hast to be returned
    pub limit: Option<i32>,
    /// The index of the last item that hast to be returned
    pub offset: i32,
}

/// Extract query parameters from the `/questions` route
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the quetions we need
/// `/questions?limit=1&offset=10`
/// # Example usage
/// ```rust
/// use std::collections::HashMap;
///
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = types::extract_pagination(query).unwrap();
/// assert_eq(Some(1), p.limit);
/// assert_eq(10, p.offset);
/// ```
pub fn extract_pagination(params: &HashMap<String, String>) -> Result<Pagination, Error> {
    match (params.get("limit"), params.get("offset")) {
        (Some(limit), Some(offset)) => Ok(Pagination {
            // Takes the "limit" parameters in the query
            // and tries to convert it to a number
            limit: Some(limit.parse().map_err(Error::ParseError)?),

            // Takes the "offset" parameters in the query
            // and tries to convert it to a number
            offset: offset.parse().map_err(Error::ParseError)?,
        }),

        _ => Err(Error::MissingParameters),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pagination() {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), "1".to_string());
        params.insert("offset".to_string(), "1".to_string());

        let pagination = extract_pagination(&params).unwrap();
        let expected = Pagination {
            limit: Some(1),
            offset: 1,
        };
        assert_eq!(expected, pagination);
    }

    #[test]
    fn missing_offset_parameter() {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), "1".to_string());

        let pagination = format!("{}", extract_pagination(&params).unwrap_err());
        let expected = format!("{}", Error::MissingParameters);
        assert_eq!(expected, pagination);
    }

    #[test]
    fn missing_limit_parameter() {
        let mut params = HashMap::new();
        params.insert("offset".to_string(), "1".to_string());

        let pagination = format!("{}", extract_pagination(&params).unwrap_err());
        let expected = format!("{}", Error::MissingParameters);
        assert_eq!(expected, pagination);
    }

    #[test]
    fn wrong_offset_type() {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), "1".to_string());
        params.insert("offset".to_string(), "NOT_A_NUMBER".to_string());

        let pagination = extract_pagination(&params).unwrap_err();
        assert!(matches!(pagination, Error::ParseError(_)));
    }

    #[test]
    fn wrong_limit_type() {
        let mut params = HashMap::new();
        params.insert("offset".to_string(), "1".to_string());
        params.insert("limit".to_string(), "NOT_A_NUMBER".to_string());

        let pagination = extract_pagination(&params).unwrap_err();
        assert!(matches!(pagination, Error::ParseError(_)));
    }
}
