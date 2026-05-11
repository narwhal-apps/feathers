pub mod api;
pub mod auth;
pub mod types;

use std::sync::OnceLock;

/// Process-wide HTTP client used for all GitHub requests (REST API + OAuth
/// device flow). Sharing one `reqwest::Client` reuses the TLS context and
/// connection pool across requests — building a fresh client per call meant
/// a new TLS handshake every time.
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub(crate) fn client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("feathers")
            .build()
            .expect("reqwest client builder should succeed")
    })
}
