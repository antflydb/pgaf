use pgrx::prelude::*;

use crate::client::{AntflyClient, SearchRequest};

/// Search an Antfly collection and return (id, score, data) tuples.
///
/// Usage:
///   SELECT * FROM antfly_search('http://localhost:8080', 'my_collection', 'search query');
///
/// Returns a set of (id TEXT, score FLOAT8, data JSONB) rows that can be
/// joined back to the source table.
#[pg_extern]
fn antfly_search(
    base_url: &str,
    collection: &str,
    query: &str,
    limit: default!(Option<i32>, "NULL"),
) -> TableIterator<'static, (name!(id, String), name!(score, f64), name!(data, pgrx::JsonB))> {
    let client = AntflyClient::new(base_url).unwrap_or_else(|e| {
        pgrx::error!("pgaf: failed to create client: {}", e);
    });

    let req = SearchRequest {
        table: collection,
        query_string: "",
        fields: vec![],
        limit: limit.map(|l| l as i64).or(Some(10)),
        end_user_search: query,
        models: vec![],
    };

    let hits = client.search(&req).unwrap_or_else(|e| {
        pgrx::error!("pgaf: search failed: {}", e);
    });

    let rows: Vec<_> = hits
        .into_iter()
        .map(|hit| {
            let id = hit
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let score = hit
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            (id, score, pgrx::JsonB(hit))
        })
        .collect();

    TableIterator::new(rows)
}

/// Retrieve the configured Antfly server URL from a GUC or table option.
/// For now, a simple helper that validates connectivity.
#[pg_extern]
fn antfly_status(base_url: &str) -> String {
    match AntflyClient::new(base_url) {
        Ok(_client) => format!("pgaf: connected to {}", base_url),
        Err(e) => format!("pgaf: error: {}", e),
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_antfly_status() {
        // Should return an error string for an unreachable server, not panic
        let result = crate::functions::antfly_status("http://127.0.0.1:19999");
        assert!(result.starts_with("pgaf:"));
    }
}
