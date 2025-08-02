#![allow(unused_crate_dependencies)]

use cfg_if::cfg_if;
use leptos::prelude::*;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_session::config::{BrowserSession, CookieContentSecurity, SessionMiddlewareBuilder};
        use std::net::SocketAddr;
        use std::sync::{Arc, Mutex};
        use clap::Parser;
        use trace_viewer::{sessions::SessionEngine, structs::{SearchStatus, Select, Topics}, shell};
        use supermusr_common::CommonKafkaOpts;
        use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};
        use tracing::info;

        #[derive(Parser)]
        #[clap(author, version, about)]
        struct Cli {
            #[clap(flatten)]
            common_kafka_options: CommonKafkaOpts,

            /// Kafka consumer group.
            #[clap(long)]
            consumer_group: String,

            #[clap(flatten)]
            topics: Topics,

            /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used.
            #[clap(long)]
            otel_endpoint: Option<String>,

            /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
            #[clap(long, default_value = "")]
            otel_namespace: String,

            /// Endpoint on which OpenMetrics flavour metrics are available.
            #[clap(long, default_value = "127.0.0.1:9090")]
            observability_address: SocketAddr,

            #[clap(flatten)]
            select: Select,

            /// Kafka timeout for polling the broker for topic info.
            /// If this feature is failing, then increasing this value may help.
            #[clap(long, default_value = "1000")]
            poll_broker_timeout_ms: u64,

            /// Name to apply to this particular instance.
            #[clap(long)]
            name: Option<String>,
        }

        #[actix_web::main]
        async fn main() -> miette::Result<()> {
            use actix_files::Files;
            use leptos_actix::{generate_route_list, LeptosRoutes};
            use miette::IntoDiagnostic;
            use trace_viewer::{App, DefaultData, sessions::SessionEngine};
            use actix_web::cookie::Key;
            use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};

            // set up logging
            console_error_panic_hook::set_once();

            //let file = File::create("../Saves/tracing.log").expect("");
            let stdout_tracer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(false);

            // This filter is applied to the stdout tracer
            let log_filter = EnvFilter::from_default_env();

            let subscriber =
                tracing_subscriber::Registry::default().with(stdout_tracer.with_filter(log_filter));

            //  This is only called once, so will never panic
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default should only be called once");

            let args = Cli::parse();

            let default = DefaultData {
                broker: args.common_kafka_options.broker.clone(),
                topics: args.topics.clone(),
                select: args.select.clone(),
                username: args.common_kafka_options.username.clone(),
                password: args.common_kafka_options.password.clone(),
                consumer_group: args.consumer_group.clone(),
                poll_broker_timeout_ms: args.poll_broker_timeout_ms.clone()
            };

            let conf = get_configuration(None).unwrap();
            let addr = conf.leptos_options.site_addr;

            let session_engine = Arc::new(Mutex::new(SessionEngine::default()));

            actix_web::HttpServer::new(move || {
                // Generate the list of routes in your Leptos App
                let routes = generate_route_list({
                    let default = default.clone();
                    move || {
                        view!{ <App /> }
                    }
                });
                let leptos_options = &conf.leptos_options;
                let site_root = leptos_options.site_root.clone().to_string();
            
                let status = Arc::new(Mutex::new(SearchStatus::Off));
                let secret_key = Key::generate();


                info!("listening on http://{}", &addr);
                actix_web::App::new()
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    .leptos_routes_with_context(routes, {
                        let default = default.clone();
                        let status = status.clone();
                        let session_engine = session_engine.clone();
                        move ||{
                            provide_context(default.clone());
                            provide_context(status.clone());
                            provide_context(session_engine.clone());
                        }
                    }, {
                        let leptos_options = leptos_options.clone();
                        let default = default.clone();
                        let session_engine = session_engine.clone();
                        move || {
                            let session_engine = session_engine.lock().expect("");
                            shell(leptos_options.clone(), default.clone())
                        }
                    })
                    .app_data(actix_web::web::Data::new(leptos_options.to_owned()))
                    .wrap(
                        SessionMiddleware::builder(
                            CookieSessionStore::default(),
                            secret_key.clone(),
                        )
                        .cookie_name(String::from("trace-viewer"))
                        .cookie_secure(false)
                        .session_lifecycle(BrowserSession::default())
                        .cookie_same_site(actix_web::cookie::SameSite::Strict)
                        .cookie_content_security(CookieContentSecurity::Signed)
                        .cookie_http_only(true)
                        .build()
                    )
            })
            .bind(&addr)
            .into_diagnostic()?
            .run()
            .await
            .into_diagnostic()
        }
    }
}

/*
#[cfg(feature = "ssr")]
pub async fn handle_sse(req: HttpRequest) -> impl actix_web::Responder {
    use std::time::Duration;
    use leptos_sse::ServerSentEvents;
    use futures::stream;

    use tokio_stream::StreamExt as _;

    let actual_status = req.app_data::<Arc<Mutex<SearchStatus>>>().expect("").clone();
    let mut current_status = SearchStatus::Off;

    let stream_item_iter = stream::repeat_with(move || {
        match actual_status.lock() {
            Ok(actual_status) => 
                if *actual_status != current_status {
                    current_status = actual_status.clone();
                    Ok(current_status.clone())
                } else {
                    Ok(current_status.clone())
                },
            Err(e) => unimplemented!(),
        }
    })
    .throttle(Duration::from_secs(1));

    let stream = ServerSentEvents::new("search_status", stream_item_iter).expect("");

    actix_web_lab::sse::Sse::from_stream(stream).with_keep_alive(Duration::from_secs(5))
}
 */
#[cfg(not(feature = "ssr"))]
fn main() {
    use console_error_panic_hook as _;
    use trace_viewer as _;
    mount_to_body(|| {
        view! {
            "Please run using SSR"
        }
    });
}
