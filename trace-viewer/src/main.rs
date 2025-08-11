#![allow(unused_crate_dependencies)]

use cfg_if::cfg_if;
use leptos::prelude::*;
use trace_viewer::structs::ClientSideData;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::net::SocketAddr;
        use std::sync::{Arc, Mutex};
        use clap::Parser;
        use trace_viewer::{structs::{DefaultData, ServerSideData, Topics}, shell};
        use supermusr_common::CommonKafkaOpts;
        use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};
        use tracing::info;
        use tokio::time::Duration;

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
            default: DefaultData,

            /// Name of the broker.
            #[clap(long)]
            broker_name: String,

            /// Optional link to the redpanda console. If present, displayed in the topbar.
            #[clap(long)]
            link_to_redpanda_console: Option<String>,

            /// Name to apply to this particular instance.
            #[clap(long)]
            name: Option<String>,
        }

        #[actix_web::main]
        async fn main() -> miette::Result<()> {
            use actix_files::Files;
            use leptos_actix::{generate_route_list, LeptosRoutes};
            use miette::IntoDiagnostic;
            use trace_viewer::{App, structs::DefaultData, sessions::SessionEngine};

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

            let server_side_data = ServerSideData {
                broker: args.common_kafka_options.broker.clone(),
                topics: args.topics.clone(),
                username: args.common_kafka_options.username.clone(),
                password: args.common_kafka_options.password.clone(),
                consumer_group: args.consumer_group.clone(),
            };

            let client_side_data = ClientSideData {
                broker_name: args.broker_name,
                link_to_redpanda_console: args.link_to_redpanda_console.clone(),
                default_data : args.default.clone()
            };

            let conf = get_configuration(None).unwrap();
            let addr = conf.leptos_options.site_addr;

            let session_engine = Arc::new(Mutex::new(SessionEngine::new(&server_side_data)));

            // Spawn the "purge expired sessions" task.
            let purge_sessions = tokio::task::spawn({
                let session_engine = session_engine.clone();
                async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(60));

                    loop {
                        interval.tick().await;
                        session_engine.lock().expect("").purge_expired();
                    }
                }
            });

            actix_web::HttpServer::new(move || {
                // Generate the list of routes in your Leptos App
                let routes = generate_route_list({
                    let client_side_data = client_side_data.clone();
                    move || {
                        provide_context(client_side_data.clone());
                        view!{ <App /> }
                    }
                });
                let leptos_options = &conf.leptos_options;
                let site_root = leptos_options.site_root.clone().to_string();

                info!("listening on http://{}", &addr);
                actix_web::App::new()
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    .leptos_routes_with_context(routes, {
                        let server_side_data = server_side_data.clone();
                        let session_engine = session_engine.clone();
                        let client_side_data = client_side_data.clone();
                        move ||{
                            provide_context(server_side_data.clone());
                            provide_context(session_engine.clone());
                            provide_context(client_side_data.clone());
                        }
                    }, {
                        let leptos_options = leptos_options.clone();
                        move || {
                            shell(leptos_options.clone())
                        }
                    })
                    .app_data(actix_web::web::Data::new(leptos_options.to_owned()))
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
