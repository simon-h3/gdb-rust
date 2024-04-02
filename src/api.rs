
#[macro_use]
extern crate actix_web;

use std::env;
use actix_web::{get, web, App, HttpServer, Responder};
use crate::disk;
/*

    GET
    /node/id
    /relationship/id

    PUT (Update)

    /node/id/info
    /relationship/id/info

    POST

    /node/newId/info
    /relationship/newId/info

    DELETE

    /node/id
    /relationship/id

 */

#[get("/node/{id}")]
async fn HTTP_get_node(id: web::Path<String>) -> Responder {
    let id = id.parse::<u64>()?;    // unsafe TODO>>
    let result = disk::get_node(id);

//     TODO: return result through HTTP...
}

#[actix_rt::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register HTTP requests handlers
            .service(HTTP_get_node);
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

