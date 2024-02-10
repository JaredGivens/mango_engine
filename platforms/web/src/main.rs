use actix_files::Files;
use actix_web::{
    middleware::{DefaultHeaders, Logger},
    App, HttpServer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(
                Files::new("/", "fe")
                    .use_hidden_files()
                    .index_file("index.html"),
            )
            .wrap(
                DefaultHeaders::new()
                    .add(("referrer-policy", "no-referrer-when-downgrade"))
                    .add(("cross-origin-opener-policy", "same-origin"))
                    .add(("cross-origin-embedder-policy", "require-corp")),
            )
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
