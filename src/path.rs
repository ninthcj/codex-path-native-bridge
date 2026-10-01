const SOURCE: &str = "/v1/codex/responses";
const DESTINATION: &str = "/v1/responses";

// The host supplies a valid HTTP URI. Locate its path without normalizing any
// bytes of the authority or query, and without allocating on unrelated requests.
pub(super) fn rewrite(method: &str, uri: &mut String) {
    if !matches!(method, "GET" | "POST") {
        return;
    }
    let end = uri.find('?').unwrap_or(uri.len());
    let before_query = &uri[..end];
    let start = if before_query.starts_with('/') {
        0
    } else if let Some((scheme, authority_and_path)) = before_query.split_once("://") {
        let Some(slash) = authority_and_path.find('/') else {
            return;
        };
        scheme.len() + 3 + slash
    } else {
        return;
    };
    if &uri[start..end] == SOURCE {
        uri.replace_range(start..end, DESTINATION);
    }
}

#[cfg(test)]
mod tests {
    use super::rewrite;

    #[test]
    fn rewrites_native_transports_preserving_query_bytes() {
        for method in ["GET", "POST"] {
            for (input, expected) in [
                ("/v1/codex/responses", "/v1/responses"),
                ("/v1/codex/responses?", "/v1/responses?"),
                (
                    "/v1/codex/responses?a=%2F&b=1&b=2&empty=&q=?/v1/codex/responses",
                    "/v1/responses?a=%2F&b=1&b=2&empty=&q=?/v1/codex/responses",
                ),
                (
                    "https://Example.COM:8443/v1/codex/responses?token=%2b+",
                    "https://Example.COM:8443/v1/responses?token=%2b+",
                ),
            ] {
                let mut uri = input.to_owned();
                rewrite(method, &mut uri);
                assert_eq!(uri, expected);
            }
        }
    }

    #[test]
    fn leaves_other_paths_and_uri_forms_unchanged() {
        for method in ["GET", "POST"] {
            for input in [
                "/v1/responses",
                "/v1/codex",
                "/v1/codex/responses/",
                "/v1/codex/responses/extra",
                "/v1/codex/responses-extra",
                "/v1/Codex/responses",
                "/v1/codex/%72esponses",
                "/v1/models?path=/v1/codex/responses",
                "https://example.com?path=/v1/codex/responses",
                "example.com:443",
                "*",
            ] {
                let mut uri = input.to_owned();
                rewrite(method, &mut uri);
                assert_eq!(uri, input);
            }
        }
    }

    #[test]
    fn leaves_other_methods_including_case_variants_unchanged() {
        for method in ["HEAD", "OPTIONS", "PUT", "DELETE", "PATCH", "CONNECT", "get", "post", "Get", "Post"] {
            let input = "/v1/codex/responses?stream=true";
            let mut uri = input.to_owned();
            rewrite(method, &mut uri);
            assert_eq!(uri, input);
        }
    }
}
