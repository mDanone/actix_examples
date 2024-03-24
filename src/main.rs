use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Mutex;
use std::time::Duration;
use serde::{Deserialize, Deserializer};


struct AppState {
    app_name: String,
    counter: Mutex<i32>
}

impl AppState {
    fn new(app_name: &str) -> Self {
        AppState {
            app_name: app_name.to_string(),
            counter: Mutex::new(0),
        }
    }
}

#[derive(Deserialize)]
struct User {
    nickname: String,
    id: u32
}


#[derive(Deserialize, Debug)]
enum ActionType {
    Sleep,
    WakeUp
}

#[derive(Deserialize, Debug)]
struct Action {
    #[serde(deserialize_with = "from_action_type")]
    action_type: Option<ActionType>
}

fn from_action_type<'de, D>(deserializer: D) -> Result<Option<ActionType>, D::Error>
    where
        D: Deserializer<'de>,
{
    let res: Option<ActionType> = Deserialize::deserialize(deserializer).unwrap_or(None);
    Ok(res)
}


#[get("/{nickname}/{id}")]
async fn hello(data: web::Data<AppState>, user: web::Path<User>, action: web::Query<Action>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok()
        .body(
            format!(
                "Hello: (nickname: {}, id: {}), AppName: {}, Action: {:?}",
                user.nickname,
                user.id,
                data.app_name,
                action.action_type
            )
        )
}

#[derive(Deserialize, Debug)]
struct SomeExampleJson {
    some_data: String
}

#[post("/echo")]
async fn echo(example_json: web::Json<SomeExampleJson>) -> impl Responder {
    HttpResponse::Ok().body(format!("{:?}", example_json.some_data))
}

async fn manual_hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello world: counted {}", data.counter.lock().unwrap()))
}

fn scoped_examples(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/app")
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let state = web::Data::new(AppState::new("Example App"));

    let mut builder = SslAcceptor::mozilla_intermediate(
        SslMethod::tls()
    ).unwrap();
    builder
        .set_private_key_file("nopass.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            .configure(scoped_examples)
            .app_data(state.clone())
    })
    .workers(4)
    .keep_alive(Duration::from_secs(10))
    .bind_openssl(("127.0.0.1", 8080), builder)?.run().await
}
