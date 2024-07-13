/*

    HTTP Handling functions as API endpoint to gdb.

*/

#[macro_use]
extern crate actix_web;

use crate::disk;
use actix_web::middleware::Logger;
use actix_web::{get, put, web, App, HttpServer, Responder};
use std::env;
/*

   GET
   /node/id            <
   /relationship/id XXXXXXXX

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

#[post("/node/")]
async fn http_post_node(incoming_node: web::Json<Node>) -> impl Responder {
    // Process the incoming node data
    println!("Creating node:\t {:?} :\t {:?}", node.id, Utc::now());

    let node: Node = node.into_inner(); // into itself...
    disk::create_node(node);

    println!("Saved node:\t {:?} :\t {:?}\n", node.id, Utc::now());

    HttpResponse::Created().json(incoming_node)
}

#[get("/node/{id}")]
async fn http_get_node(id: web::Path<String>) -> impl Responder {
    let id = id.parse::<u64>()?; // unsafe TODO>>
    let result = disk::get_node_from_id(id);

    //     TODO: return result through HTTP...
}

// #[get("/relationship/{id}")]
// async fn http_get_relationship(id: web::Path<String>) -> impl Responder {
//     let id = id.parse::<u64>()?;    // unsafe TODO>>
//     let result = disk::get_relationship_from_id(id);
//
// //     TODO: return result through HTTP...
// }

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    println!("-- Starting Server --");

    HttpServer::new(|| {
        App::new()
            // enable logger - always register actix-web Logger middleware last
            .wrap(Logger::default())
            // register HTTP requests handlers
            .service(http_get_node);
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
