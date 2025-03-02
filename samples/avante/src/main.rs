// This is the main file of the project
// SAMLE API PROJECT RESTFUL API
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

// Item構造体は、APIで使用されるアイテムのデータを表します。
#[derive(Serialize, Deserialize)]
struct Item {
    id: u32,
    name: String,
}

// get_items関数は、すべてのアイテムを取得して返します。
async fn get_items() -> impl Responder {
    let items = vec![
        Item {
            id: 1,
            name: "Item 1".to_string(),
        },
        Item {
            id: 2,
            name: "Item 2".to_string(),
        },
    ];
    HttpResponse::Ok().json(items)
}

// get_item関数は、指定されたIDのアイテムを取得して返します。
async fn get_item(path: web::Path<u32>) -> impl Responder {
    let item = Item {
        id: path.into_inner(),
        name: "Item".to_string(),
    };
    HttpResponse::Ok().json(item)
}

// create_item関数は、新しいアイテムを作成します。
async fn create_item(item: web::Json<Item>) -> impl Responder {
    HttpResponse::Created().json(item.into_inner())
}

// update_item関数は、指定されたIDのアイテムを更新します。
async fn update_item(path: web::Path<u32>, item: web::Json<Item>) -> impl Responder {
    let mut updated_item = item.into_inner();
    updated_item.id = path.into_inner();
    HttpResponse::Ok().json(updated_item)
}

// delete_item関数は、指定されたIDのアイテムを削除します。
async fn delete_item(path: web::Path<u32>) -> impl Responder {
    HttpResponse::NoContent().finish()
}

// main関数は、アプリケーションのエントリーポイントです。
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/items", web::get().to(get_items))
            .route("/items/{id}", web::get().to(get_item))
            .route("/items", web::post().to(create_item))
            .route("/items/{id}", web::put().to(update_item))
            .route("/items/{id}", web::delete().to(delete_item))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
