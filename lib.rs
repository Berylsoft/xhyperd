pub type TsSecs = u64;
pub type TsNanos = u32;

#[derive(serde::Serialize)]
pub struct RawTime {
    // dir: bool,
    secs: TsSecs,
    nanos: TsNanos,
}

impl From<std::time::SystemTime> for RawTime {
    fn from(value: std::time::SystemTime) -> Self {
        let res = value.duration_since(std::time::UNIX_EPOCH);
        let dir = res.is_ok();
        assert!(dir, "time before unix epoch");
        let dur = res.unwrap_or_else(|err| err.duration());
        let secs = dur.as_secs();
        let nanos = dur.subsec_nanos();
        Self { secs, nanos }
    }
}

pub struct IncomingRequest {
    pub time: std::time::SystemTime,
    pub remote_addr: std::net::SocketAddr,
    pub header: http::request::Parts,
    pub payload: bytes::Bytes,
}

#[derive(serde::Serialize)]
pub struct RawRequest {
    // pub id: TimeBasedId,
    pub time: RawTime,
    pub remote_addr: std::net::SocketAddr,
    #[serde(with = "http_serde::method")]
    pub method: http::Method,
    #[serde(with = "http_serde::uri")]
    pub uri: http::Uri,
    #[serde(with = "http_serde::version")]
    pub version: http::Version,
    #[serde(with = "http_serde::header_map")]
    pub headers: http::HeaderMap,
    pub payload: bytes::Bytes,
}

impl From<IncomingRequest> for RawRequest {
    fn from(IncomingRequest { time, remote_addr, header, payload }: IncomingRequest) -> Self {
        let http::request::Parts { method, uri, version, headers, extensions, .. } = header;
        assert!(extensions.is_empty(), "request has additional data");
        let time: RawTime = time.into();
        // let id = time.time_based_id();
        Self { time, remote_addr, method, version, uri, headers, payload }
    }
}
