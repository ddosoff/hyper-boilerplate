#![feature(await_macro, async_await, futures_api)]

mod settings;
mod tls;

// 0.3 futures
use futures::future::{FutureExt, TryFutureExt};
use hyper::{
    // Spawn server need it
    rt::Future,

    // This function turns a closure which returns a future into an
    // implementation of the the Hyper `Service` trait, which is an
    // asynchronous function from a generic `Request` to a `Response`.
    service::service_fn,

    // Miscellaneous types from Hyper for working with HTTP.
    Body,
    Request,
    Response,
    Server,
};
use log::{info, warn};
use std::{io::Write, thread, time};

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Always return successfully with a response containing a body with
    // a friendly greeting ;)
    Ok(Response::new(Body::from("hello, world!")))
}

// Can't use traits to spawn multithread or singlethread tokio::runtime..
macro_rules! spawn_servers {
    ($settings:expr, $rt:expr, $exec:expr) => {
        // httpS config exists?
        if let Some(https) = $settings.https {
            let tls_settings = tls::read(&https.cert_pem, &https.key_rsa_pem);

            info!(
                "Binding HTTPS({}, {}): {} ...",
                https.cert_pem,
                https.key_rsa_pem,
                https.host_port.to_string()
            );

            // Clonable thread safe rustls server settings
            let tls_settings = std::sync::Arc::new(tls_settings);
            let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_settings);

            let mut incoming =
                hyper::server::conn::AddrIncoming::bind(&https.host_port).expect("Can't bind");

            incoming.set_nodelay(true);

            use hyper::rt::Stream; // incoming future trait
            let incoming = incoming
                .and_then(move |socket| tls_acceptor.accept(socket))
                .then(|tls_stream| match tls_stream {
                    Ok(x) => Ok::<_, std::io::Error>(Some(x)),
                    Err(_e) => {
                        // Errors could be handled here
                        warn!("TLS handshake error: {}", _e);
                        Ok(None)
                        //Err(_e)
                    }
                })
                // This magic I do not understand..
                .filter_map(|x| x);

            // Create a server bound on the provided address
            let serve_future = Server::builder(incoming)
                .executor($exec.clone())
                // Serve requests using our `async serve_req` function.
                // `serve` takes a closure which returns a type implementing the
                // `Service` trait. `service_fn` returns a value implementing the
                // `Service` trait, and accepts a closure which goes from request
                // to a future of the response. In order to use our `serve_req`
                // function with Hyper, we have to box it and put it in a compatability
                // wrapper to go from a futures 0.3 future (the kind returned by
                // `async fn`) to a futures 0.1 future (the kind used by Hyper).
                .serve(|| service_fn(|req| serve_req(req).boxed().compat()))
                .map_err(move |e| warn!("HTTPS server {} error {}", &https.host_port, &e));

            // Add server future to list
            $rt.spawn(serve_future);
        }

        if let Some(http) = $settings.http {
            info!("Binding HTTP: {} ...", http.host_port.to_string());
            let serve_future = Server::bind(&http.host_port)
                .executor($exec.clone())
                // Serve requests using our `async serve_req` function.
                // `serve` takes a closure which returns a type implementing the
                // `Service` trait. `service_fn` returns a value implementing the
                // `Service` trait, and accepts a closure which goes from request
                // to a future of the response. In order to use our `serve_req`
                // function with Hyper, we have to box it and put it in a compatability
                // wrapper to go from a futures 0.3 future (the kind returned by
                // `async fn`) to a futures 0.1 future (the kind used by Hyper).
                .serve(|| service_fn(|req| serve_req(req).boxed().compat()))
                .map_err(move |e| warn!("HTTP server {} error {}", &http.host_port, &e));

            $rt.spawn(serve_future);
        }
    };
}

fn hello_from_ferris() -> Result<(), std::io::Error> {
    let out = b"Hello fellow ChatBros!";
    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());

    ferris_says::say(out, out.len(), &mut writer)?;

    // just add empty line
    writeln!(writer)?;

    writer.flush()
}

fn spawn_single_threaded_in_separate_thread(settings: settings::Settings) {
    // Start single threaded event loop
    thread::Builder::new()
        .name("single_threaded".to_string())
        .spawn(move || {
            let mut rt = tokio::runtime::current_thread::Runtime::new()
                .expect("Can't create single thread runtime");

            // Impossible to get like rt.executor() !?
            // let exec = rt.executor();
            let exec = tokio::runtime::current_thread::TaskExecutor::current();

            info!("** Spawning single threaded servers **");
            spawn_servers!(settings, rt, exec);
            info!("");

            // Wait all spawned futures ready..
            rt.run().expect("Single threaded runtime error");

            eprintln!("Single threaded done!");
        })
        .expect("Can't spawn single threaded");
}

fn spawn_multi_threaded_and_wait(settings: settings::Settings) {
    // Start multi threaded event loops
    let mut rt = tokio::runtime::Runtime::new().expect("Can't create multi threaded runtime");

    let exec = rt.executor();

    info!("** Spawning multithreaded servers **");
    spawn_servers!(settings, rt, exec);
    info!("");

    rt.shutdown_on_idle()
        .wait()
        .expect("Multithreaded runtime run error");
}

fn main() {
    hello_from_ferris().unwrap();

    // Show logging info by default
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();

    // Mutable, because will increment ports for multithreaded runtime
    let mut settings = settings::Settings::read();

    spawn_single_threaded_in_separate_thread(settings.clone());

    // Sleep until first child thread print logs
    thread::sleep(time::Duration::from_millis(10));

    settings.increment_ports();

    spawn_multi_threaded_and_wait(settings);
}
