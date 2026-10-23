//! worker が Web を読むための取得（SSRF 対策つき）。
//!
//! CMS のスクリプトは dify-sandbox（seccomp でネットワークを禁止）で動くので Web を読めない。
//! サンドボックスは開けず（LAN への SSRF になる）、worker が許可したホストだけを取得する。
//!
//! - HTTPS（443 番）だけ。URL に認証情報（`user:pass@`）・IP アドレスのホストは使えない
//! - ホストは CMS の `fetch_allowed_hosts`（`enabled` のもの）と完全一致すること。環境変数では設定しない
//! - DNS を引き、返った **すべて** のアドレスが公開アドレスであること（ループバック・プライベート・
//!   リンクローカル（メタデータの 169.254.169.254 を含む）・ユニークローカル・CGNAT などは拒否）。
//!   接続は確かめたアドレスに固定する（reqwest の `resolve`）ので、DNS rebinding で別のアドレスに繋がらない。
//!   プロキシの環境変数も使わない（`no_proxy`）
//! - リダイレクトは自動で追わず、1 回ごとに URL・許可リスト・DNS を確かめ直す（最大 [`FetchLimits::max_redirects`] 回）
//! - タイムアウト・本文の大きさの上限（超えたら読むのをやめて失敗）
//! - JSON はそのまま（ただしキーが `_html` で終わる文字列は読みやすいテキストにする）、HTML は見出しと段落の
//!   テキストにし、[`FetchLimits::max_text_chars`] 文字で切る
//! - span（`web.fetch`）にはホスト・状態コード・バイト数だけを入れ、本文は入れない

use super::cms_client::{CmsClient, ListQuery};
use crate::trace_context::record_error;
use anyhow::{bail, Context};
use async_trait::async_trait;
use reqwest::Url;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tracing::{field::Empty, Instrument};

/// 取得してよいホストの一覧の CMS のコレクション
pub(crate) const FETCH_ALLOWED_HOSTS: &str = "fetch_allowed_hosts";

/// 一覧を 1 度に取得する数（ホストはこれより少ない前提）
const ALLOWED_HOSTS_LIMIT: u32 = 200;

const HTTPS_PORT: u16 = 443;

const USER_AGENT: &str = "botcast-worker (+https://github.com/wakame-tech/botcast)";

/// 取得の上限
#[derive(Debug, Clone)]
pub(crate) struct FetchLimits {
    /// 本文のバイト数の上限（超えたら失敗）
    pub max_body_bytes: usize,
    /// 返すテキストの文字数の上限（超えたら切る）
    pub max_text_chars: usize,
    pub max_redirects: usize,
    pub connect_timeout: Duration,
    pub timeout: Duration,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            max_body_bytes: 2 * 1024 * 1024,
            max_text_chars: 20_000,
            max_redirects: 3,
            connect_timeout: Duration::from_secs(5),
            timeout: Duration::from_secs(20),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContentKind {
    Json,
    Html,
    Text,
}

impl ContentKind {
    fn from_content_type(content_type: &str) -> Option<Self> {
        let mime = content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if mime == "application/json" || mime.ends_with("+json") {
            Some(Self::Json)
        } else if mime == "text/html" || mime == "application/xhtml+xml" {
            Some(Self::Html)
        } else if mime.starts_with("text/") {
            Some(Self::Text)
        } else {
            None
        }
    }
}

/// 取得した内容（LLM・スクリプトに渡せる形にしたもの）
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FetchedContent {
    /// リダイレクトを追ったあとの URL
    pub url: String,
    pub content_type: String,
    pub kind: ContentKind,
    pub text: String,
    /// [`FetchLimits::max_text_chars`] で切ったか
    pub truncated: bool,
}

/// URL を取得するもの（本番は [`SafeFetcher`]、テストはフェイク）
#[async_trait]
pub(crate) trait UrlFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> anyhow::Result<FetchedContent>;
}

/// 取得してよいホストの一覧（本番は CMS の `fetch_allowed_hosts`）
#[async_trait]
pub(crate) trait HostAllowlist: Send + Sync {
    async fn allowed_hosts(&self) -> anyhow::Result<Vec<String>>;
}

#[async_trait]
impl HostAllowlist for CmsClient {
    async fn allowed_hosts(&self) -> anyhow::Result<Vec<String>> {
        let records = self
            .list_records(
                FETCH_ALLOWED_HOSTS,
                &ListQuery {
                    limit: Some(ALLOWED_HOSTS_LIMIT),
                    ..Default::default()
                },
            )
            .await
            .context("Failed to list allowed hosts")?;
        Ok(hosts_from_records(&records))
    }
}

/// `fetch_allowed_hosts` のレコードから `enabled` のホストを取り出す
pub(crate) fn hosts_from_records(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .filter(|r| r["data"]["enabled"].as_bool() == Some(true))
        .filter_map(|r| r["data"]["host"].as_str())
        .map(normalize_host)
        .filter(|h| !h.is_empty())
        .collect()
}

fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

/// DNS（本番は OS のリゾルバー、テストはフェイク）
#[async_trait]
pub(crate) trait Resolver: Send + Sync {
    async fn resolve(&self, host: &str, port: u16) -> anyhow::Result<Vec<SocketAddr>>;
}

pub(crate) struct SystemResolver;

#[async_trait]
impl Resolver for SystemResolver {
    async fn resolve(&self, host: &str, port: u16) -> anyhow::Result<Vec<SocketAddr>> {
        Ok(tokio::net::lookup_host((host, port))
            .await
            .with_context(|| format!("Failed to resolve {host}"))?
            .collect())
    }
}

/// 1 回の GET の結果（リダイレクトは追わない）
#[derive(Debug, Clone, Default)]
pub(crate) struct RawResponse {
    pub status: u16,
    pub location: Option<String>,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

/// 1 回の GET（本番は reqwest、テストはフェイク）。`addr` に固定して接続し、リダイレクトは追わない
#[async_trait]
pub(crate) trait Transport: Send + Sync {
    async fn get(
        &self,
        url: &Url,
        addr: SocketAddr,
        limits: &FetchLimits,
    ) -> anyhow::Result<RawResponse>;
}

pub(crate) struct ReqwestTransport;

#[async_trait]
impl Transport for ReqwestTransport {
    async fn get(
        &self,
        url: &Url,
        addr: SocketAddr,
        limits: &FetchLimits,
    ) -> anyhow::Result<RawResponse> {
        let host = url.host_str().context("URL has no host")?;
        // 接続先を確かめたアドレスに固定する（reqwest は自分で DNS を引かない）。
        // TLS の SNI・証明書の検証・Host ヘッダーは URL のホストのまま
        let client = reqwest::Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(limits.connect_timeout)
            .timeout(limits.timeout)
            .user_agent(USER_AGENT)
            .resolve(host, addr)
            .build()
            .context("Failed to build HTTP client")?;
        let mut response = client
            .get(url.clone())
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/html;q=0.9, text/plain;q=0.8",
            )
            .send()
            .await
            .with_context(|| format!("Failed to fetch {host}"))?;
        let status = response.status().as_u16();
        let header = |name: reqwest::header::HeaderName| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let location = header(reqwest::header::LOCATION);
        let content_type = header(reqwest::header::CONTENT_TYPE);
        // リダイレクト・エラーの本文は使わないので読まない
        let body = if (200..300).contains(&status) {
            let content_length = response.content_length();
            read_capped(&mut response, content_length, limits.max_body_bytes).await?
        } else {
            vec![]
        };
        Ok(RawResponse {
            status,
            location,
            content_type,
            body,
        })
    }
}

/// 本文を少しずつ読むもの（本番は reqwest の応答、テストはフェイク）
#[async_trait]
pub(crate) trait ChunkSource: Send {
    async fn next_chunk(&mut self) -> anyhow::Result<Option<Vec<u8>>>;
}

#[async_trait]
impl ChunkSource for reqwest::Response {
    async fn next_chunk(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(self
            .chunk()
            .await
            .context("Failed to read response body")?
            .map(|b| b.to_vec()))
    }
}

/// 本文を `max` バイトまで読む。`Content-Length` が上限を超えるなら読まずに、
/// 読んでいる途中で超えたらそこでやめて失敗する
pub(crate) async fn read_capped(
    source: &mut dyn ChunkSource,
    content_length: Option<u64>,
    max: usize,
) -> anyhow::Result<Vec<u8>> {
    if content_length.is_some_and(|len| len > max as u64) {
        bail!("response body is too large (Content-Length over {max} bytes)");
    }
    let mut body = Vec::new();
    while let Some(chunk) = source.next_chunk().await? {
        if body.len() + chunk.len() > max {
            bail!("response body is too large (over {max} bytes)");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// URL を確かめる: HTTPS（443 番）・認証情報なし・ドメイン名のホストで、許可リストと完全一致すること
pub(crate) fn check_url(url: &str, allowed_hosts: &[String]) -> anyhow::Result<Url> {
    let url = Url::parse(url.trim()).with_context(|| format!("invalid URL: {url}"))?;
    if url.scheme() != "https" {
        bail!("only https URLs can be fetched: {url}");
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("URLs with credentials cannot be fetched");
    }
    let host = match url.host() {
        Some(url::Host::Domain(d)) => normalize_host(d),
        Some(_) => bail!("URLs with an IP address host cannot be fetched: {url}"),
        None => bail!("URL has no host: {url}"),
    };
    if url.port_or_known_default() != Some(HTTPS_PORT) {
        bail!("only port {HTTPS_PORT} can be fetched: {url}");
    }
    if !allowed_hosts.contains(&host) {
        bail!(
            "host {host} is not in the allowed hosts (CMS collection '{FETCH_ALLOWED_HOSTS}')"
        );
    }
    Ok(url)
}

/// DNS の結果をすべて確かめ、接続するアドレスを選ぶ（1 つでも公開アドレスでなければ拒否）
pub(crate) fn pick_addr(host: &str, addrs: &[SocketAddr]) -> anyhow::Result<SocketAddr> {
    let Some(first) = addrs.first() else {
        bail!("{host} did not resolve to any address");
    };
    if let Some(bad) = addrs.iter().find(|a| !is_public_ip(a.ip())) {
        // アドレスはログにだけ出し、エラー（LLM・ジョブの結果に入る）には入れない
        tracing::warn!("refusing to fetch {host}: resolved to non-public address {}", bad.ip());
        bail!("{host} resolves to a non-public address");
    }
    Ok(*first)
}

/// 公開（グローバル）アドレスか
pub(crate) fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_ipv4(v4),
        IpAddr::V6(v6) => is_public_ipv6(v6),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        // 169.254.0.0/16（クラウドのメタデータ 169.254.169.254 を含む）
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // 0.0.0.0/8
        || o[0] == 0
        // CGNAT 100.64.0.0/10
        || (o[0] == 100 && (o[1] & 0xc0) == 64)
        // IETF のプロトコル割り当て 192.0.0.0/24
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        // ベンチマーク 198.18.0.0/15
        || (o[0] == 198 && (o[1] & 0xfe) == 18)
        // 予約 240.0.0.0/4
        || o[0] >= 240)
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    // IPv4 射影（::ffff:a.b.c.d）は中の IPv4 で判断する
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_public_ipv4(v4);
    }
    let s = ip.segments();
    let embedded_v4 = |hi: u16, lo: u16| {
        Ipv4Addr::new((hi >> 8) as u8, hi as u8, (lo >> 8) as u8, lo as u8)
    };
    // グローバルユニキャスト 2000::/3 以外（::1・::・fc00::/7・fe80::/10・ff00::/8・IPv4 互換など）は拒否
    if (s[0] & 0xe000) != 0x2000 {
        // NAT64 64:ff9b::/96 は中の IPv4 で判断する
        if s[0] == 0x64 && s[1] == 0xff9b && s[2..6] == [0, 0, 0, 0] {
            return is_public_ipv4(embedded_v4(s[6], s[7]));
        }
        return false;
    }
    // 2001::/32（Teredo）・2001:db8::/32（文書用）・2001:2::/48（ベンチマーク）は拒否
    if s[0] == 0x2001 && (s[1] == 0 || s[1] == 0x0db8 || (s[1] == 2 && s[2] == 0)) {
        return false;
    }
    // 6to4 2002::/16 は中の IPv4 で判断する
    if s[0] == 0x2002 {
        return is_public_ipv4(embedded_v4(s[1], s[2]));
    }
    true
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

/// 許可リスト・DNS の確認・接続先の固定・リダイレクトの確認をして取得する
pub(crate) struct SafeFetcher {
    allowlist: Arc<dyn HostAllowlist>,
    resolver: Arc<dyn Resolver>,
    transport: Arc<dyn Transport>,
    limits: FetchLimits,
}

impl SafeFetcher {
    /// 本番の構成（OS のリゾルバー・reqwest・既定の上限）
    pub(crate) fn new(allowlist: Arc<dyn HostAllowlist>) -> Self {
        Self::with_parts(
            allowlist,
            Arc::new(SystemResolver),
            Arc::new(ReqwestTransport),
            FetchLimits::default(),
        )
    }

    pub(crate) fn with_parts(
        allowlist: Arc<dyn HostAllowlist>,
        resolver: Arc<dyn Resolver>,
        transport: Arc<dyn Transport>,
        limits: FetchLimits,
    ) -> Self {
        Self {
            allowlist,
            resolver,
            transport,
            limits,
        }
    }

    async fn fetch_inner(&self, url: &str, span: &tracing::Span) -> anyhow::Result<FetchedContent> {
        let hosts = self.allowlist.allowed_hosts().await?;
        let mut current = check_url(url, &hosts)?;
        let mut redirects = 0;
        loop {
            let host = current.host_str().context("URL has no host")?.to_string();
            span.record("server.address", host.as_str());
            let addrs = self.resolver.resolve(&host, HTTPS_PORT).await?;
            let addr = pick_addr(&host, &addrs)?;
            let raw = self.transport.get(&current, addr, &self.limits).await?;
            span.record("http.response.status_code", raw.status);
            if is_redirect(raw.status) {
                if redirects >= self.limits.max_redirects {
                    bail!("too many redirects (over {})", self.limits.max_redirects);
                }
                redirects += 1;
                span.record("fetch.redirects", redirects);
                let location = raw
                    .location
                    .with_context(|| format!("redirect from {host} has no Location"))?;
                let next = current
                    .join(&location)
                    .with_context(|| format!("invalid redirect location from {host}"))?;
                // リダイレクト先も最初の URL と同じ確認をする（別のホストなら許可リストに必要）
                current = check_url(next.as_str(), &hosts)?;
                continue;
            }
            if !(200..300).contains(&raw.status) {
                bail!("{host} returned HTTP {}", raw.status);
            }
            span.record("http.response.body.size", raw.body.len());
            return to_content(current, raw, &self.limits);
        }
    }
}

#[async_trait]
impl UrlFetcher for SafeFetcher {
    async fn fetch(&self, url: &str) -> anyhow::Result<FetchedContent> {
        let span = tracing::info_span!(
            "web.fetch",
            otel.kind = "client",
            server.address = Empty,
            http.response.status_code = Empty,
            http.response.body.size = Empty,
            fetch.redirects = 0,
            otel.status_code = Empty,
            otel.status_message = Empty,
        );
        let result = self
            .fetch_inner(url, &span)
            .instrument(span.clone())
            .await;
        if let Err(e) = &result {
            record_error(&span, &format!("{e:#}"));
        }
        result
    }
}

/// 応答を LLM・スクリプトに渡すテキストにする
fn to_content(url: Url, raw: RawResponse, limits: &FetchLimits) -> anyhow::Result<FetchedContent> {
    let content_type = raw.content_type.unwrap_or_default();
    let kind = ContentKind::from_content_type(&content_type)
        .with_context(|| format!("unsupported content type {content_type:?}"))?;
    let body = String::from_utf8_lossy(&raw.body);
    let text = match kind {
        ContentKind::Json => json_to_text(&body),
        ContentKind::Html => html_to_text(&body),
        ContentKind::Text => body.into_owned(),
    };
    let (text, truncated) = truncate_chars(&text, limits.max_text_chars);
    Ok(FetchedContent {
        url: url.to_string(),
        content_type,
        kind,
        text,
        truncated,
    })
}

/// JSON はそのまま返す。キーが `_html` で終わる文字列（Zenn の `body_html` など）だけは
/// HTML のままだと長くタグばかりになるので、読みやすいテキストにして JSON に戻す
fn json_to_text(body: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(body) else {
        return body.to_string();
    };
    if !reduce_html_fields(&mut value) {
        return body.to_string();
    }
    serde_json::to_string(&value).unwrap_or_else(|_| body.to_string())
}

/// キーが `_html` で終わる文字列をテキストにする。変えたら true
fn reduce_html_fields(value: &mut Value) -> bool {
    match value {
        Value::Object(obj) => {
            let mut changed = false;
            for (k, v) in obj.iter_mut() {
                if let (true, Value::String(s)) = (k.ends_with("_html"), &*v) {
                    *v = Value::String(html_to_text(s));
                    changed = true;
                } else {
                    changed |= reduce_html_fields(v);
                }
            }
            changed
        }
        Value::Array(items) => {
            // すべての要素を変えるので、途中で止めない（`any` は使わない）
            let mut changed = false;
            for v in items.iter_mut() {
                changed |= reduce_html_fields(v);
            }
            changed
        }
        _ => false,
    }
}

/// HTML を見出しと段落のテキストにする（script・style・画像・コードブロックは除く）
pub(crate) fn html_to_text(html: &str) -> String {
    let md = readable_text::ReadableText::extract(html).unwrap_or_default();
    // 行末の空白を除き、3 行以上の空行を 1 行にまとめる
    let mut out = String::with_capacity(md.len());
    let mut blank = 0;
    for line in md.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

/// `max` 文字までにする。切ったら true
fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    match s.char_indices().nth(max) {
        Some((i, _)) => (s[..i].to_string(), true),
        None => (s.to_string(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn hosts(h: &[&str]) -> Vec<String> {
        h.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn check_url_allows_only_listed_https_hosts() {
        let allowed = hosts(&["zenn.dev"]);
        assert!(check_url("https://zenn.dev/api/articles?order=weekly&count=5", &allowed).is_ok());
        assert!(check_url("https://ZENN.dev./api", &allowed).is_ok());
        assert!(check_url("https://zenn.dev:443/api", &allowed).is_ok());
        for (url, want) in [
            ("http://zenn.dev/api", "only https"),
            ("ftp://zenn.dev/a", "only https"),
            ("https://evil.example/api", "not in the allowed hosts"),
            // サブドメイン・似たドメインは別のホスト
            ("https://api.zenn.dev/a", "not in the allowed hosts"),
            ("https://zenn.dev.evil.example/a", "not in the allowed hosts"),
            ("https://zenn.dev:8443/api", "only port 443"),
            ("https://user:pass@zenn.dev/api", "credentials"),
            ("https://127.0.0.1/", "IP address"),
            ("https://[::1]/", "IP address"),
            // 10 進表記の IP も URL の解析で IPv4 になる
            ("https://2130706433/", "IP address"),
            ("not a url", "invalid URL"),
        ] {
            let err = format!("{:#}", check_url(url, &allowed).unwrap_err());
            assert!(err.contains(want), "{url}: {err}");
        }
        assert!(check_url("https://zenn.dev/", &[]).is_err());
    }

    #[test]
    fn hosts_from_records_takes_enabled_hosts() {
        let records = vec![
            json!({ "id": "a", "data": { "host": "Zenn.dev", "enabled": true } }),
            json!({ "id": "b", "data": { "host": "off.example", "enabled": false } }),
            json!({ "id": "c", "data": { "host": "noflag.example" } }),
        ];
        assert_eq!(hosts_from_records(&records), hosts(&["zenn.dev"]));
    }

    #[test]
    fn rejects_non_public_addresses() {
        for ip in [
            "127.0.0.1", "10.1.2.3", "172.16.0.1", "172.31.255.255", "192.168.1.1",
            "169.254.169.254", "100.64.0.1", "100.127.255.255", "0.0.0.0", "0.1.2.3",
            "255.255.255.255", "224.0.0.1", "240.0.0.1", "192.0.0.1", "198.18.0.1",
            "192.0.2.1", "::1", "::", "fc00::1", "fd00:ec2::254", "fe80::1", "fec0::1",
            "ff02::1", "::ffff:127.0.0.1", "::ffff:169.254.169.254", "::127.0.0.1",
            "64:ff9b::a9fe:a9fe", "2002:7f00:1::1", "2001:db8::1", "2001::1",
        ] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "8.8.8.8", "1.1.1.1", "172.32.0.1", "100.128.0.1", "34.117.59.81",
            "2606:4700::1111", "::ffff:8.8.8.8", "64:ff9b::808:808", "2002:808:808::1",
        ] {
            assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
        }
    }

    fn sa(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn pick_addr_requires_every_address_to_be_public() {
        assert_eq!(pick_addr("h", &[sa("8.8.8.8:443")]).unwrap(), sa("8.8.8.8:443"));
        // 1 つでも内部のアドレスが混ざれば拒否する（どれに繋がるか分からないため）
        let err = pick_addr("h", &[sa("8.8.8.8:443"), sa("10.0.0.1:443")]).unwrap_err();
        assert!(err.to_string().contains("non-public"), "{err}");
        // エラーにはアドレスを入れない
        assert!(!err.to_string().contains("10.0.0.1"), "{err}");
        assert!(pick_addr("h", &[]).is_err());
    }

    struct FakeAllowlist(Vec<String>);

    #[async_trait]
    impl HostAllowlist for FakeAllowlist {
        async fn allowed_hosts(&self) -> anyhow::Result<Vec<String>> {
            Ok(self.0.clone())
        }
    }

    struct FakeResolver(HashMap<String, Vec<SocketAddr>>);

    #[async_trait]
    impl Resolver for FakeResolver {
        async fn resolve(&self, host: &str, _port: u16) -> anyhow::Result<Vec<SocketAddr>> {
            self.0.get(host).cloned().context("NXDOMAIN")
        }
    }

    /// URL ごとの応答を返し、呼ばれた URL と接続先を記録する
    #[derive(Default)]
    struct FakeTransport {
        responses: HashMap<String, RawResponse>,
        calls: Mutex<Vec<(String, SocketAddr)>>,
    }

    #[async_trait]
    impl Transport for FakeTransport {
        async fn get(
            &self,
            url: &Url,
            addr: SocketAddr,
            _limits: &FetchLimits,
        ) -> anyhow::Result<RawResponse> {
            self.calls.lock().unwrap().push((url.to_string(), addr));
            self.responses.get(url.as_str()).cloned().context("no fake response")
        }
    }

    fn ok(content_type: &str, body: &str) -> RawResponse {
        RawResponse {
            status: 200,
            content_type: Some(content_type.to_string()),
            body: body.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    fn redirect(location: &str) -> RawResponse {
        RawResponse {
            status: 302,
            location: Some(location.to_string()),
            ..Default::default()
        }
    }

    fn fetcher(
        allowed: &[&str],
        dns: &[(&str, &[&str])],
        responses: Vec<(&str, RawResponse)>,
    ) -> (SafeFetcher, Arc<FakeTransport>) {
        let transport = Arc::new(FakeTransport {
            responses: responses
                .into_iter()
                .map(|(u, r)| (u.to_string(), r))
                .collect(),
            ..Default::default()
        });
        let resolver = FakeResolver(
            dns.iter()
                .map(|(h, addrs)| (h.to_string(), addrs.iter().map(|a| sa(a)).collect()))
                .collect(),
        );
        let fetcher = SafeFetcher::with_parts(
            Arc::new(FakeAllowlist(hosts(allowed))),
            Arc::new(resolver),
            transport.clone(),
            FetchLimits {
                max_text_chars: 50,
                ..Default::default()
            },
        );
        (fetcher, transport)
    }

    #[tokio::test]
    async fn fetches_json_as_is_and_connects_to_the_checked_address() {
        let (f, t) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443"])],
            vec![("https://zenn.dev/api/articles", ok("application/json; charset=utf-8", r#"{"articles": [ 1 ]}"#))],
        );
        let content = f.fetch("https://zenn.dev/api/articles").await.unwrap();
        assert_eq!(content.kind, ContentKind::Json);
        assert_eq!(content.text, r#"{"articles": [ 1 ]}"#);
        assert!(!content.truncated);
        assert_eq!(
            *t.calls.lock().unwrap(),
            vec![("https://zenn.dev/api/articles".to_string(), sa("34.117.59.81:443"))]
        );
    }

    #[tokio::test]
    async fn refuses_hosts_resolving_to_private_addresses_without_connecting() {
        // DNS rebinding: 許可したホストが内部のアドレスを返す
        let (f, t) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443", "169.254.169.254:443"])],
            vec![("https://zenn.dev/", ok("text/plain", "x"))],
        );
        let err = f.fetch("https://zenn.dev/").await.unwrap_err();
        assert!(format!("{err:#}").contains("non-public"), "{err:#}");
        assert!(t.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn refuses_hosts_not_in_the_allowlist() {
        let (f, t) = fetcher(&["zenn.dev"], &[("evil.example", &["8.8.8.8:443"])], vec![]);
        assert!(f.fetch("https://evil.example/").await.is_err());
        assert!(t.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn follows_redirects_only_to_checked_urls() {
        // 同じホスト内の相対リダイレクトは追う
        let (f, _) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443"])],
            vec![
                ("https://zenn.dev/a", redirect("/b")),
                ("https://zenn.dev/b", ok("text/plain", "done")),
            ],
        );
        let content = f.fetch("https://zenn.dev/a").await.unwrap();
        assert_eq!(content.text, "done");
        assert_eq!(content.url, "https://zenn.dev/b");

        // 許可していないホスト・http・内部のアドレスに解決されるホストへのリダイレクトは拒否する
        for (location, want) in [
            ("https://evil.example/x", "not in the allowed hosts"),
            ("http://zenn.dev/x", "only https"),
            ("https://169.254.169.254/latest/meta-data", "IP address"),
            ("https://internal.zenn.dev/x", "non-public"),
        ] {
            let (f, t) = fetcher(
                &["zenn.dev", "internal.zenn.dev"],
                &[
                    ("zenn.dev", &["34.117.59.81:443"]),
                    ("internal.zenn.dev", &["10.0.0.5:443"]),
                ],
                vec![("https://zenn.dev/a", redirect(location))],
            );
            let err = format!("{:#}", f.fetch("https://zenn.dev/a").await.unwrap_err());
            assert!(err.contains(want), "{location}: {err}");
            // リダイレクト先には接続しない
            assert_eq!(t.calls.lock().unwrap().len(), 1, "{location}");
        }
    }

    #[tokio::test]
    async fn limits_the_number_of_redirects() {
        let (f, t) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443"])],
            vec![("https://zenn.dev/loop", redirect("/loop"))],
        );
        let err = format!("{:#}", f.fetch("https://zenn.dev/loop").await.unwrap_err());
        assert!(err.contains("too many redirects"), "{err}");
        assert_eq!(t.calls.lock().unwrap().len(), FetchLimits::default().max_redirects + 1);
    }

    #[tokio::test]
    async fn fails_on_http_errors_and_unsupported_types() {
        let (f, _) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443"])],
            vec![
                ("https://zenn.dev/404", RawResponse { status: 404, ..Default::default() }),
                ("https://zenn.dev/img", ok("image/png", "\u{89}PNG")),
            ],
        );
        assert!(format!("{:#}", f.fetch("https://zenn.dev/404").await.unwrap_err()).contains("HTTP 404"));
        assert!(format!("{:#}", f.fetch("https://zenn.dev/img").await.unwrap_err()).contains("unsupported content type"));
    }

    #[tokio::test]
    async fn html_is_reduced_to_text_and_truncated() {
        let html = "<html><head><title>t</title><script>alert(1)</script></head><body>\
            <h1>見出し</h1><p>段落の<a href='x'>リンク</a>です。</p><img src='a.png'>\
            <p>あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん</p></body></html>";
        let (f, _) = fetcher(
            &["zenn.dev"],
            &[("zenn.dev", &["34.117.59.81:443"])],
            vec![("https://zenn.dev/p", ok("text/html; charset=utf-8", html))],
        );
        let content = f.fetch("https://zenn.dev/p").await.unwrap();
        assert_eq!(content.kind, ContentKind::Html);
        assert!(content.text.contains("見出し"), "{}", content.text);
        assert!(content.text.contains("段落のリンクです。"), "{}", content.text);
        assert!(!content.text.contains("alert"), "{}", content.text);
        assert!(!content.text.contains('<'), "{}", content.text);
        assert!(content.truncated);
        assert_eq!(content.text.chars().count(), 50);
    }

    #[test]
    fn json_html_fields_are_reduced_to_text() {
        let body = r#"{"article":{"title":"T","body_html":"<h2>章</h2><p>本文</p><script>x()</script>"}}"#;
        let v: Value = serde_json::from_str(&json_to_text(body)).unwrap();
        assert_eq!(v["article"]["title"], "T");
        let text = v["article"]["body_html"].as_str().unwrap();
        assert!(text.contains("章") && text.contains("本文"), "{text}");
        assert!(!text.contains('<') && !text.contains("x()"), "{text}");
        // JSON でなければそのまま
        assert_eq!(json_to_text("not json"), "not json");
    }

    struct Chunks(Vec<Vec<u8>>, usize);

    #[async_trait]
    impl ChunkSource for Chunks {
        async fn next_chunk(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
            self.1 += 1;
            Ok(if self.0.is_empty() { None } else { Some(self.0.remove(0)) })
        }
    }

    #[tokio::test]
    async fn read_capped_enforces_the_size_limit() {
        let mut src = Chunks(vec![b"abc".to_vec(), b"de".to_vec()], 0);
        assert_eq!(read_capped(&mut src, None, 5).await.unwrap(), b"abcde");

        // 読んでいる途中で超えたら、残りを読まずに失敗する
        let mut src = Chunks(vec![b"abc".to_vec(), b"def".to_vec(), b"ghi".to_vec()], 0);
        let err = read_capped(&mut src, None, 5).await.unwrap_err();
        assert!(err.to_string().contains("too large"), "{err}");
        assert_eq!(src.1, 2);

        // Content-Length が上限を超えるなら読まない
        let mut src = Chunks(vec![b"abc".to_vec()], 0);
        assert!(read_capped(&mut src, Some(6), 5).await.is_err());
        assert_eq!(src.1, 0);
    }

    /// 実物の zenn.dev から取得する（DNS・接続先の固定・TLS を確かめる）
    #[tokio::test]
    #[ignore = "requires network access to zenn.dev"]
    async fn fetch_against_zenn() {
        let fetcher = SafeFetcher::new(Arc::new(FakeAllowlist(hosts(&["zenn.dev"]))));
        let list = fetcher
            .fetch("https://zenn.dev/api/articles?order=weekly&count=5")
            .await
            .unwrap();
        assert_eq!(list.kind, ContentKind::Json);
        let v: Value = serde_json::from_str(&list.text).unwrap();
        let slug = v["articles"][0]["slug"].as_str().unwrap();
        let article = fetcher
            .fetch(&format!("https://zenn.dev/api/articles/{slug}"))
            .await
            .unwrap();
        assert!(!article.text.contains("<p>"), "body_html はテキストになる");
        println!("{} chars, truncated: {}", article.text.chars().count(), article.truncated);
    }

    #[test]
    fn truncate_chars_counts_characters() {
        assert_eq!(truncate_chars("あいう", 2), ("あい".to_string(), true));
        assert_eq!(truncate_chars("あいう", 3), ("あいう".to_string(), false));
    }
}
