use ::config::Config;
use actix_cors::Cors;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use actix_web::web::{Bytes, Query};
use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use zen_engine::DecisionEngine;
use zen_engine::loader::{FilesystemLoader, FilesystemLoaderOptions};

use crate::main_config::MainConfig;

mod main_config;

#[derive(Debug, Default, Serialize, Clone)]
struct Info {
    message: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
struct RuleInfo {
    rule: Option<String>
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(Info {
        message: "Welcome to Zen Engine!".to_string()
    })
}

#[post("/execute_rule")]
async fn execute_rule(data: web::Data<MainConfig>, bytes: Bytes, query: Query<RuleInfo>) -> impl Responder {
    let main_config = data.get_ref();
    let engine = DecisionEngine::new(FilesystemLoader::new(
        FilesystemLoaderOptions {
            keep_in_memory: true, // optionally, keep in memory for increase performance
            root: main_config.rules_folder.clone(),
        }));
    let rule_json = match &query.rule { Some(r) => r.clone(), None => String::from("test_rule.json") };
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => {
            match serde_json::from_str(text.as_str()) {
                Ok(context) => {
                    println!("context: {}", context);
                    // If you plan on using it multiple times, you may cache JDM for minor performance gains
                    // In case of bindings (in other languages, this increase is much greater)
                    let promotion_decision = engine.get_decision(rule_json.as_str())
                        .await.unwrap();
                    let result = promotion_decision.evaluate(&context)
                        .await.unwrap();
                    HttpResponse::Ok().json(result.result)
                }
                Err(e) => handle_error(e),
            }
        }
        Err(e) => {
            println!("An error occurred");
            HttpResponse::Ok().json(Info {
                message: format!("{:?}", e)
            })
        }
    }
}

fn handle_error(e: serde_json::error::Error) -> HttpResponse {
    println!("An error occurred");
    HttpResponse::Ok().json(Info {
        message: format!("{:?}", e)
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();
    let config: MainConfig = config_.try_deserialize().unwrap();
    let server_addr = config.server_addr.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(config.clone()))
            .service(index)
            .service(execute_rule)
    })
        .bind(server_addr)?
        .run()
        .await
}
