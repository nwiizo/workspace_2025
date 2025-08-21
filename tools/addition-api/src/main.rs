use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct AdditionRequest {
    a: f64,
    b: f64,
}

#[derive(Deserialize, Serialize)]
struct AdditionResponse {
    result: f64,
}

async fn add(data: web::Json<AdditionRequest>) -> impl Responder {
    let result = data.a + data.b;
    HttpResponse::Ok().json(AdditionResponse { result })
}

#[derive(Deserialize, Serialize)]
struct SubtractionRequest {
    a: f64,
    b: f64,
}

#[derive(Deserialize, Serialize)]
struct SubtractionResponse {
    result: f64,
}

async fn sub(data: web::Json<SubtractionRequest>) -> impl Responder {
    let result = data.a - data.b;
    HttpResponse::Ok().json(SubtractionResponse { result })
}

#[derive(Deserialize, Serialize)]
struct MultiplicationRequest {
    a: f64,
    b: f64,
}

#[derive(Deserialize, Serialize)]
struct MultiplicationResponse {
    result: f64,
}

async fn mul(data: web::Json<MultiplicationRequest>) -> impl Responder {
    let result = data.a * data.b;
    HttpResponse::Ok().json(MultiplicationResponse { result })
}

#[derive(Deserialize, Serialize)]
struct DivisionRequest {
    a: f64,
    b: f64,
}

#[derive(Deserialize, Serialize)]
struct DivisionResponse {
    result: f64,
}

async fn div(data: web::Json<DivisionRequest>) -> impl Responder {
    if data.b == 0.0 {
        return HttpResponse::BadRequest().body("Division by zero is not allowed");
    }
    let result = data.a / data.b;
    HttpResponse::Ok().json(DivisionResponse { result })
}
#[derive(Deserialize, Serialize)]
struct SquareRequest {
    a: f64,
}

#[derive(Deserialize, Serialize)]
struct SquareResponse {
    result: f64,
}

async fn square(data: web::Json<SquareRequest>) -> impl Responder {
    let result = data.a * data.a;
    HttpResponse::Ok().json(SquareResponse { result })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/add").route(web::post().to(add)))
            .service(web::resource("/sub").route(web::post().to(sub)))
            .service(web::resource("/mul").route(web::post().to(mul)))
            .service(web::resource("/div").route(web::post().to(div)))
            .service(web::resource("/square").route(web::post().to(square)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_add() {
        let mut app = test::init_service(
            App::new().service(web::resource("/add").route(web::post().to(add))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/add")
            .set_json(&AdditionRequest { a: 1.0, b: 2.0 })
            .to_request();
        let resp: AdditionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 3.0);
    }

    #[actix_web::test]
    async fn test_sub() {
        let mut app = test::init_service(
            App::new().service(web::resource("/sub").route(web::post().to(sub))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/sub")
            .set_json(&SubtractionRequest { a: 5.0, b: 3.0 })
            .to_request();
        let resp: SubtractionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 2.0);
    }

    #[actix_web::test]
    async fn test_mul() {
        let mut app = test::init_service(
            App::new().service(web::resource("/mul").route(web::post().to(mul))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/mul")
            .set_json(&MultiplicationRequest { a: 2.0, b: 3.0 })
            .to_request();
        let resp: MultiplicationResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 6.0);
    }

    #[actix_web::test]
    async fn test_div() {
        let mut app = test::init_service(
            App::new().service(web::resource("/div").route(web::post().to(div))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/div")
            .set_json(&DivisionRequest { a: 6.0, b: 2.0 })
            .to_request();
        let resp: DivisionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 3.0);
    }

    #[actix_web::test]
    async fn test_div_by_zero() {
        let mut app = test::init_service(
            App::new().service(web::resource("/div").route(web::post().to(div))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/div")
            .set_json(&DivisionRequest { a: 1.0, b: 0.0 })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_square() {
        let mut app = test::init_service(
            App::new().service(web::resource("/square").route(web::post().to(square))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/square")
            .set_json(&SquareRequest { a: 3.0 })
            .to_request();
        let resp: SquareResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 9.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_add() {
        let mut app = test::init_service(
            App::new().service(web::resource("/add").route(web::post().to(add))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/add")
            .set_json(&AdditionRequest { a: 1.0, b: 2.0 })
            .to_request();
        let resp: AdditionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 3.0);
    }

    #[actix_web::test]
    async fn test_sub() {
        let mut app = test::init_service(
            App::new().service(web::resource("/sub").route(web::post().to(sub))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/sub")
            .set_json(&SubtractionRequest { a: 5.0, b: 3.0 })
            .to_request();
        let resp: SubtractionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 2.0);
    }

    #[actix_web::test]
    async fn test_mul() {
        let mut app = test::init_service(
            App::new().service(web::resource("/mul").route(web::post().to(mul))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/mul")
            .set_json(&MultiplicationRequest { a: 2.0, b: 3.0 })
            .to_request();
        let resp: MultiplicationResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 6.0);
    }

    #[actix_web::test]
    async fn test_div() {
        let mut app = test::init_service(
            App::new().service(web::resource("/div").route(web::post().to(div))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/div")
            .set_json(&DivisionRequest { a: 6.0, b: 2.0 })
            .to_request();
        let resp: DivisionResponse = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp.result, 3.0);
    }

    #[actix_web::test]
    async fn test_div_by_zero() {
        let mut app = test::init_service(
            App::new().service(web::resource("/div").route(web::post().to(div))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/div")
            .set_json(&DivisionRequest { a: 1.0, b: 0.0 })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 400);
    }
}
