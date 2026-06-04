use reqwest::Client;

/// Build an HTTP client that respects HTTP_PROXY/HTTPS_PROXY/ALL_PROXY environment variables.
/// Falls back to a direct connection if no proxy is configured.
pub fn build_client(user_agent: &str) -> Client {
    let mut builder = Client::builder().user_agent(user_agent);

    // reqwest automatically reads HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars
    // when the system's TLS backend is used. With rustls-tls (our case),
    // we need to explicitly check and set the proxy.
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("ALL_PROXY"))
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("http_proxy"))
        .or_else(|_| std::env::var("all_proxy"))
    {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build().expect("Failed to build HTTP client")
}
