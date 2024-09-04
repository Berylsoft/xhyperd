// use actor_core::Context;

// trait HttpContext: Context {
//     type Req: xhyperd::RawRequest;
// }

/*
HttpContext: filter & executor
filter: &RawRequest -> bool; static(fn) & dynamic(event to send)
executor: &RawRequest -> RawResponse
*/

async fn _main() {
    env_logger::init();
    // TODO how axum with hyper1 do shutdown
    let (tx, rx) = async_channel::bounded::<()>(1);

    async_global_executor::spawn(async move {
        // let ctx = actor::spawn(empowerd::GlobalContext::init(empowerd::GlobalConfig {
        //     root: &std::path::PathBuf::from(&"C:/swap/dftest"),
        //     db_mem_max: None,
        //     api_path: None,
        // }));

        // TODO(async-net) AsyncToSocketAddrs
        let listener = async_net::TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], 10027))).await.unwrap();
        loop {
            let fut = futures_lite::future::or(async {
                Some(listener.accept().await)
            }, async {
                // error also means to close
                let res = rx.recv().await;
                log::info!("close signal received: {:?}", res);
                None
            });
            if let Some(res) = fut.await {
                let (stream, remote_addr) = res.unwrap();
                let io = smol_hyper::rt::FuturesIo::new(stream);
                // let local_ref = ctx.clone();

                async_global_executor::spawn(async move {
                    let service_fn = move |req: hyper::Request<hyper::body::Incoming>| {
                        // let local_ref = local_ref.clone();
                        async move {
                            let (header, payload) = req.into_parts();
                            let raw_req = xhyperd::RawRequest::from(xhyperd::IncomingRequest {
                                time: std::time::SystemTime::now(),
                                remote_addr,
                                header,
                                payload: http_body_util::BodyExt::collect(payload).await.unwrap().to_bytes(),
                            });
                            let raw_req_json = serde_json::to_string(&raw_req).unwrap();
                            println!("{raw_req_json}");
                            let res = xhyperd::RawResponse {
                                time: std::time::SystemTime::now(),
                                status: http::StatusCode::OK,
                                version: Default::default(),
                                headers: http::HeaderMap::new(),
                                body_log_type: xhyperd::BodyLogType::Full,
                                body: bytes::Bytes::from(raw_req_json),
                            };
                            let (log_res, body) = res.to_log();
                            let log_res_json = serde_json::to_string(&log_res).unwrap();
                            println!("{log_res_json}");
                            let http_res = log_res.build(body);
                            Ok::<_, core::convert::Infallible>(http_res)
                        }
                    };
                    let service = hyper::service::service_fn(service_fn);
                    let conn = hyper::server::conn::http1::Builder::new().serve_connection(io, service);
                    // std::pin::pin!(conn).graceful_shutdown()
                    conn.await.unwrap();
                }).detach();
            } else {
                // TODO still not "ctx closed" randomly
                // ctx.wait_close().await.unwrap();
                break;
            }
        }
    }).detach();

    async_ctrlc::CtrlC::new().unwrap().await;
    tx.send(()).await.unwrap();
    log::info!("close signal sent");
}

fn main() {
    async_global_executor::block_on(_main());
}
