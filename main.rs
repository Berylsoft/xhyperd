// use actor_core::Context;

// trait HttpContext: Context {
//     type Req: xhyperd::RawRequest;
// }

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
                    let service = hyper::service::service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                        // let local_ref = local_ref.clone();
                        async move {
                            let (header, payload) = req.into_parts();
                            println!("{}", &serde_json::to_string(&xhyperd::RawRequest::from(xhyperd::IncomingRequest {
                                time: std::time::SystemTime::now(),
                                remote_addr,
                                header,
                                payload: http_body_util::BodyExt::collect(payload).await.unwrap().to_bytes(),
                            })).unwrap());
                            Ok::<_, core::convert::Infallible>(http::Response::builder().body(String::new()).unwrap())
                        }
                    });
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
