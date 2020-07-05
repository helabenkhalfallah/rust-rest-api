use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::env;
use user_service::UserService;

mod user_router;
mod user_service;

pub struct ServiceManager {
    user: UserService,
}

impl ServiceManager {
    pub fn new(user: UserService) -> Self {
        ServiceManager { user }
    }
}

pub struct AppState {
    service_manager: ServiceManager,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // init env
    dotenv().ok();

    // init logger middleware
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // server url
    let server_url = env::var("SERVER_URL").expect("SERVER_URL is not set in .env file");

    // Parse a connection string into an options struct.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let client_options = ClientOptions::parse(&database_url).unwrap();

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Get a handle to a database.
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME is not set in .env file");
    let db = client.database(&database_name);

    // Get a handle to a collection in the database.
    let user_collection_name =
        env::var("USER_COLLECTION_NAME").expect("USER_COLLECTION_NAME is not set in .env file");
    let user_collection = db.collection(&user_collection_name);

    HttpServer::new(move || {
        let user_service_worker = UserService::new(user_collection.clone());
        let service_manager = ServiceManager::new(user_service_worker);

        // launch http server
        App::new()
            .wrap(middleware::Logger::default())
            // https://github.com/actix/examples/blob/8dab533b40d9d0640e5c75922c9e8e292ed4a7d5/sqlx_todo/src/main.rs#L41
            // pass database pool to application so we can access it inside handlers
            .data(AppState { service_manager })
            .configure(user_router::init)
    })
    .bind(server_url)?
    .run()
    .await
}
