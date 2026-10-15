use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Blocked protocol '{0}': only http and https are allowed")]
    BlockedProtocol(String),

    #[error("SSRF security violation: IP address '{0}' is private or reserved")]
    BlockedIp(IpAddr),

    #[error("DNS resolution failed for host '{0}': {1}")]
    DnsResolutionFailed(String, String),

    #[error("Asset exceeds maximum allowed size of {0} bytes")]
    OversizedAsset(u64),

    #[error("Too many redirects: exceeded limit of {0}")]
    TooManyRedirects(usize),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("IO error while saving asset: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for SSRF-safe fetcher.
#[derive(Debug, Clone)]
pub struct FetcherConfig {
    pub max_asset_bytes: u64,
    pub max_redirects: usize,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub allow_loopback_for_testing: bool,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            max_asset_bytes: 500 * 1024 * 1024, // 500 MB
            max_redirects: 5,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            allow_loopback_for_testing: false,
        }
    }
}

/// SSRF-safe asset fetcher that validates host IPs before connecting,
/// enforces redirect limits, and caps maximum download sizes.
#[derive(Clone)]
pub struct SafeFetcher {
    config: FetcherConfig,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl SafeFetcher {
    pub fn new(config: FetcherConfig) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none()) // We handle redirects manually with SSRF checks
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { config, client }
    }
}

impl Default for SafeFetcher {
    fn default() -> Self {
        Self::new(FetcherConfig::default())
    }
}

impl SafeFetcher {

    /// Validates whether an IP address is private, loopback, link-local, or reserved.
    pub fn is_ip_blocked(&self, ip: IpAddr) -> bool {
        if self.config.allow_loopback_for_testing && ip.is_loopback() {
            return false;
        }

        match ip {
            IpAddr::V4(ipv4) => {
                // Loopback: 127.0.0.0/8
                if ipv4.is_loopback() {
                    return true;
                }
                // Private networks: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                if ipv4.is_private() {
                    return true;
                }
                // Link-local: 169.254.0.0/16 (AWS/GCP/Azure instance metadata!)
                if ipv4.is_link_local() {
                    return true;
                }
                // Broadcast: 255.255.255.255
                if ipv4.is_broadcast() {
                    return true;
                }
                // Documentation / test: 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
                if ipv4.is_documentation() {
                    return true;
                }
                let octets = ipv4.octets();
                // 0.0.0.0/8
                if octets[0] == 0 {
                    return true;
                }
                // Multicast: 224.0.0.0/4
                if octets[0] >= 224 && octets[0] <= 239 {
                    return true;
                }
                // Reserved: 240.0.0.0/4
                if octets[0] >= 240 {
                    return true;
                }
                false
            }
            IpAddr::V6(ipv6) => {
                // Loopback: ::1
                if ipv6.is_loopback() {
                    return true;
                }
                // Unspecified: ::
                if ipv6.is_unspecified() {
                    return true;
                }
                // IPv4-mapped IPv6 addresses (::ffff:x.x.x.x)
                if let Some(mapped_v4) = ipv6.to_ipv4_mapped() {
                    return self.is_ip_blocked(IpAddr::V4(mapped_v4));
                }
                let segments = ipv6.segments();
                // Unique Local: fc00::/7 (fc00... or fd00...)
                if (segments[0] & 0xfe00) == 0xfc00 {
                    return true;
                }
                // Link-Local Unicast: fe80::/10
                if (segments[0] & 0xffc0) == 0xfe80 {
                    return true;
                }
                // Multicast: ff00::/8
                if (segments[0] & 0xff00) == 0xff00 {
                    return true;
                }
                // Documentation: 2001:db8::/32
                if segments[0] == 0x2001 && segments[1] == 0x0db8 {
                    return true;
                }
                false
            }
        }
    }

    /// Validates a URL and all resolved IP addresses against SSRF rules,
    /// returning the parsed URL, validated host string, and pinned SocketAddr.
    pub async fn resolve_and_validate(&self, target_url: &str) -> Result<(Url, String, std::net::SocketAddr), FetchError> {
        let parsed = Url::parse(target_url).map_err(|e| FetchError::InvalidUrl(e.to_string()))?;

        // Scheme check: only http and https allowed
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(FetchError::BlockedProtocol(scheme.to_string()));
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| FetchError::InvalidUrl("Missing host in URL".to_string()))?
            .to_string();

        let port = parsed.port_or_known_default().unwrap_or(if scheme == "https" { 443 } else { 80 });

        // Direct IP check or DNS resolution
        let socket_addr = if let Ok(ip) = host.parse::<IpAddr>() {
            if self.is_ip_blocked(ip) {
                return Err(FetchError::BlockedIp(ip));
            }
            std::net::SocketAddr::new(ip, port)
        } else {
            // Resolve host through DNS
            let addr_str = format!("{}:{}", host, port);
            let addrs = tokio::net::lookup_host(&addr_str)
                .await
                .map_err(|e| FetchError::DnsResolutionFailed(host.clone(), e.to_string()))?;

            let mut chosen = None;
            for addr in addrs {
                if self.is_ip_blocked(addr.ip()) {
                    return Err(FetchError::BlockedIp(addr.ip()));
                }
                if chosen.is_none() {
                    chosen = Some(addr);
                }
            }

            chosen.ok_or_else(|| {
                FetchError::DnsResolutionFailed(host.clone(), "No IP addresses resolved".to_string())
            })?
        };

        Ok((parsed, host, socket_addr))
    }

    /// Validates a URL and all resolved IP addresses against SSRF rules.
    pub async fn validate_url(&self, target_url: &str) -> Result<Url, FetchError> {
        let (url, _, _) = self.resolve_and_validate(target_url).await?;
        Ok(url)
    }

    /// Builds a pinned reqwest client that connects directly to the validated socket address for host.
    pub fn build_pinned_client(&self, host: &str, addr: std::net::SocketAddr) -> Result<reqwest::Client, FetchError> {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(self.config.connect_timeout)
            .timeout(self.config.request_timeout)
            .resolve(host, addr)
            .build()
            .map_err(|e| FetchError::HttpError(e.to_string()))
    }

    /// Fetches an asset from a URL into the specified destination file path.
    /// Safely handles redirects, pins DNS to pre-validated socket addresses,
    /// and enforces maximum download limits.
    pub async fn fetch_to_file(&self, url_str: &str, dest_path: &Path) -> Result<u64, FetchError> {
        let (mut current_url, mut current_host, mut current_addr) = self.resolve_and_validate(url_str).await?;
        let mut redirects_followed = 0;

        let response = loop {
            let client = self.build_pinned_client(&current_host, current_addr)?;
            let res = client
                .get(current_url.as_str())
                .send()
                .await
                .map_err(|e| FetchError::HttpError(e.to_string()))?;

            let status = res.status();
            if status.is_redirection() {
                redirects_followed += 1;
                if redirects_followed > self.config.max_redirects {
                    return Err(FetchError::TooManyRedirects(self.config.max_redirects));
                }

                let loc = res
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or_else(|| FetchError::HttpError("Redirect missing Location header".to_string()))?
                    .to_str()
                    .map_err(|e| FetchError::HttpError(e.to_string()))?;

                // Join relative redirect URLs against the current URL
                let next_url = current_url
                    .join(loc)
                    .map_err(|e| FetchError::InvalidUrl(e.to_string()))?;

                // Validate new destination and pin its IP
                let (v_url, v_host, v_addr) = self.resolve_and_validate(next_url.as_str()).await?;
                current_url = v_url;
                current_host = v_host;
                current_addr = v_addr;
                continue;
            }

            if !status.is_success() {
                return Err(FetchError::HttpError(format!(
                    "HTTP {} from {}",
                    status, current_url
                )));
            }

            break res;
        };

        // Check Content-Length if present
        if let Some(content_length) = response.content_length() {
            if content_length > self.config.max_asset_bytes {
                return Err(FetchError::OversizedAsset(self.config.max_asset_bytes));
            }
        }

        // Stream body to file with strict byte cap
        let mut file = File::create(dest_path).await?;
        let mut stream = response.bytes_stream();
        let mut total_bytes = 0u64;

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| FetchError::HttpError(e.to_string()))?;
            total_bytes += chunk.len() as u64;
            if total_bytes > self.config.max_asset_bytes {
                let _ = tokio::fs::remove_file(dest_path).await;
                return Err(FetchError::OversizedAsset(self.config.max_asset_bytes));
            }
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(total_bytes)
    }
}
