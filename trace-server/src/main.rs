#![allow(unused_crate_dependencies)]

use cfg_if::cfg_if;
use leptos::prelude::*;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::net::SocketAddr;
        use clap::Parser;
        use trace_server::{structs::{Select, Topics}, shell};
        use supermusr_common::CommonKafkaOpts;

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

            /// Interval for refreshing the app.
            #[clap(long, default_value = "100")]
            update_app_ms: u64,

            /// Interval for refreshing the app.
            #[clap(long, default_value = "1")]
            update_search_engine_ns: u64,
        }

        #[actix_web::get("./style.css")]
        async fn css() -> impl actix_web::Responder {
            println!("Getting CSS");
            actix_files::NamedFile::open_async("style.css").await
        }

        #[actix_web::main]
        async fn main() -> miette::Result<()> {
            use actix_files::Files;
            use leptos_actix::{generate_route_list, LeptosRoutes};
            use miette::IntoDiagnostic;
            use trace_server::{App, DefaultData};

            // set up logging
            //_ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            let args = Cli::parse();

            /*let consumer = supermusr_common::create_default_consumer(
                &args.common_kafka_options.broker,
                &args.common_kafka_options.username,
                &args.common_kafka_options.password,
                &args.consumer_group,
                None,
            ).into_diagnostic()?;*/

            let default = DefaultData {
                broker: args.common_kafka_options.broker.clone(),
                topics: args.topics.clone(),
                select: args.select.clone()
            };

            let conf = get_configuration(None).unwrap();
            let addr = conf.leptos_options.site_addr;

            provide_context(default);

            actix_web::HttpServer::new(move || {
                // Generate the list of routes in your Leptos App
                let routes = generate_route_list(App);
                let leptos_options = &conf.leptos_options;
                let site_root = leptos_options.site_root.clone().to_string();

                println!("listening on http://{}", &addr);

                actix_web::App::new()
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    .leptos_routes(routes, {
                        let leptos_options = leptos_options.clone();
                        move ||  shell(leptos_options.clone())
                    })
                    .service(css)
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

#[cfg(not(feature = "ssr"))]
fn main() {
    use trace_server as _;
    use console_error_panic_hook as _;
    mount_to_body(|| {
        view! {
            "Please run using SSR"
        }
    });
}