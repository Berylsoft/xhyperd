use bytes::Bytes;

pub type TsSecs = u64;
pub type TsNanos = u32;

#[derive(serde::Serialize)]
pub struct RawTime {
    dir: bool,
    secs: TsSecs,
    nanos: TsNanos,
}

impl From<std::time::SystemTime> for RawTime {
    fn from(value: std::time::SystemTime) -> Self {
        let res = value.duration_since(std::time::UNIX_EPOCH);
        let dir = res.is_ok();
        // assert!(dir, "time before unix epoch");
        let dur = res.unwrap_or_else(|err| err.duration());
        let secs = dur.as_secs();
        let nanos = dur.subsec_nanos();
        Self { dir, secs, nanos }
    }
}

pub struct IncomingRequest {
    pub time: std::time::SystemTime,
    pub remote_addr: core::net::SocketAddr,
    pub header: http::request::Parts,
    pub payload: Bytes,
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
    pub payload: Bytes,
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

pub enum BodyLogType {
    Full,
    Hash,
}

pub enum Body {
    Full(Bytes),
    // Stream(Box<dyn AsyncRead>),
    // for file, TODO ref http-body-util & other file server impl
}

#[derive(serde::Serialize)]
pub enum BodyLog {
    Full(Bytes),
    Hash([u8; 32]),
}

pub struct RawResponse {
    // pub id: TimeBasedId,
    pub time: std::time::SystemTime,
    pub status: http::StatusCode,
    pub version: http::Version,
    pub headers: http::HeaderMap,
    pub body_log_type: BodyLogType,
    pub body: Bytes,
}

#[derive(serde::Serialize)]
pub struct LogResponse {
    // pub id: TimeBasedId,
    pub time: RawTime,
    #[serde(with = "http_serde::status_code")]
    pub status: http::StatusCode,
    #[serde(with = "http_serde::version")]
    pub version: http::Version,
    #[serde(with = "http_serde::header_map")]
    pub headers: http::HeaderMap,
    pub body_log: BodyLog,
}

impl RawResponse {
    pub fn to_log(self) -> (LogResponse, Body) {
        let RawResponse { time, status, version, headers, body_log_type, body } = self;
        assert!(matches!(body_log_type, BodyLogType::Full));
        let time = time.into();
        (LogResponse {
            time,
            status,
            version,
            headers,
            body_log: BodyLog::Full(body.clone()),
        }, Body::Full(body))
    }
}

impl LogResponse {
    // TODO proper signature
    pub fn build(self, body: Body) -> http::Response<impl http_body::Body<Data = impl Send, Error = core::convert::Infallible>> {
        let Body::Full(body) = body;
        let LogResponse { status, version, headers, .. } = self;
        let mut headers = headers; // TODO
        let mut res = http::Response::builder()
            .status(status)
            .version(version);
        core::mem::swap(res.headers_mut().unwrap(), &mut headers); // TODO
        let body = http_body_util::Full::new(body);
        res.body(body).unwrap()
    }
}
