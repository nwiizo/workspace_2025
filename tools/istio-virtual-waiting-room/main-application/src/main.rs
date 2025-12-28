use axum::{
    extract::Query,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize)]
struct Product {
    id: i32,
    name: String,
    description: String,
    price: String,
    stock: i32,
}

#[derive(Debug, Deserialize)]
struct PageParams {
    page: Option<usize>,
    message: Option<String>,
}

const PRODUCTS_PER_PAGE: usize = 10;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(handle_main))
        .route("/shop", get(handle_main))
        .route("/health", get(handle_health));

    info!("Main Application is running on port {}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn handle_main(Query(params): Query<PageParams>) -> Html<String> {
    let page = params.page.unwrap_or(1);
    let message = params.message;
    let api_url = env::var("PRODUCT_API_URL")
        .unwrap_or_else(|_| "http://product-api".to_string());

    let products_html = match fetch_products(&api_url).await {
        Ok(products) => {
            let total_products = products.len();
            let total_pages = (total_products + PRODUCTS_PER_PAGE - 1) / PRODUCTS_PER_PAGE;
            let start = (page - 1) * PRODUCTS_PER_PAGE;
            let end = std::cmp::min(start + PRODUCTS_PER_PAGE, total_products);

            let page_products: Vec<_> = products.into_iter().skip(start).take(PRODUCTS_PER_PAGE).collect();

            if page_products.is_empty() {
                "<p class='no-products'>現在販売中の商品はありません。</p>".to_string()
            } else {
                let mut html = format!(
                    r#"<p class="showing">{} - {} / {} 件の商品を表示中</p>
                    <div class="products">"#,
                    start + 1,
                    end,
                    total_products
                );

                for product in page_products {
                    let stock_class = if product.stock == 0 { "sold-out" } else { "" };
                    let button = if product.stock > 0 {
                        format!(
                            r#"<button class="buy-btn" onclick="purchase({})">購入する</button>"#,
                            product.id
                        )
                    } else {
                        r#"<button class="buy-btn sold-out" disabled>売り切れ</button>"#.to_string()
                    };

                    html.push_str(&format!(
                        r#"<div class="product {}">
                            <h3>{}</h3>
                            <p class="description">{}</p>
                            <p class="price">¥{}</p>
                            <p class="stock">残り {} 点</p>
                            {}
                        </div>"#,
                        stock_class, product.name, product.description, product.price, product.stock, button
                    ));
                }

                html.push_str("</div>");

                if total_pages > 1 {
                    html.push_str(r#"<div class="pagination">"#);
                    for p in 1..=total_pages {
                        if p == page {
                            html.push_str(&format!(r#"<span class="current">{}</span>"#, p));
                        } else {
                            html.push_str(&format!(r#"<a href="?page={}">{}</a>"#, p, p));
                        }
                    }
                    html.push_str("</div>");
                }

                html
            }
        }
        Err(e) => {
            error!("Failed to fetch products: {:?}", e);
            "<p class='error'>商品の読み込みに失敗しました。しばらくしてから再度お試しください。</p>".to_string()
        }
    };

    let message_html = message
        .map(|m| format!(r#"<div class="message">{}</div>"#, m))
        .unwrap_or_default();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Limited Edition Shop</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 10px;
            margin-bottom: 30px;
            text-align: center;
        }}
        h1 {{ margin: 0; font-size: 2.5em; }}
        .success {{ color: #4CAF50; margin-top: 10px; }}
        .message {{
            background: #4CAF50;
            color: white;
            padding: 15px;
            border-radius: 5px;
            margin-bottom: 20px;
            text-align: center;
        }}
        .showing {{
            color: #666;
            margin-bottom: 20px;
        }}
        .products {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 20px;
        }}
        .product {{
            background: white;
            border-radius: 10px;
            padding: 20px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            transition: transform 0.2s;
        }}
        .product:hover {{
            transform: translateY(-5px);
        }}
        .product.sold-out {{
            opacity: 0.6;
        }}
        .product h3 {{
            margin: 0 0 10px 0;
            color: #333;
        }}
        .description {{
            color: #666;
            font-size: 0.9em;
            margin-bottom: 15px;
        }}
        .price {{
            font-size: 1.5em;
            color: #e91e63;
            font-weight: bold;
            margin: 10px 0;
        }}
        .stock {{
            color: #4CAF50;
            font-size: 0.9em;
            margin-bottom: 15px;
        }}
        .buy-btn {{
            width: 100%;
            padding: 12px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 1em;
            transition: opacity 0.2s;
        }}
        .buy-btn:hover:not(:disabled) {{
            opacity: 0.9;
        }}
        .buy-btn.sold-out {{
            background: #ccc;
            cursor: not-allowed;
        }}
        .pagination {{
            display: flex;
            justify-content: center;
            gap: 10px;
            margin-top: 30px;
        }}
        .pagination a, .pagination .current {{
            padding: 10px 15px;
            border-radius: 5px;
            text-decoration: none;
        }}
        .pagination a {{
            background: white;
            color: #667eea;
        }}
        .pagination .current {{
            background: #667eea;
            color: white;
        }}
        .error, .no-products {{
            color: #f44336;
            text-align: center;
            padding: 40px;
            background: white;
            border-radius: 10px;
        }}
        #loading {{
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0,0,0,0.5);
            justify-content: center;
            align-items: center;
            z-index: 1000;
        }}
        #loading.show {{
            display: flex;
        }}
        .spinner {{
            width: 50px;
            height: 50px;
            border: 4px solid white;
            border-top-color: #667eea;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }}
        @keyframes spin {{
            to {{ transform: rotate(360deg); }}
        }}
    </style>
</head>
<body>
    <header>
        <h1>Limited Edition Shop</h1>
        <p class="success">ショップへようこそ！</p>
    </header>
    {}
    {}
    <div id="loading">
        <div class="spinner"></div>
    </div>
    <script>
        async function purchase(productId) {{
            document.getElementById('loading').classList.add('show');
            try {{
                const response = await fetch('/api/products/' + productId + '/purchase', {{
                    method: 'POST'
                }});
                const data = await response.json();
                if (data.success) {{
                    location.href = '/?message=' + encodeURIComponent(data.message);
                }} else {{
                    alert(data.message);
                    location.reload();
                }}
            }} catch (e) {{
                alert('エラーが発生しました。');
                location.reload();
            }}
        }}
    </script>
</body>
</html>"#,
        message_html, products_html
    );

    Html(html)
}

async fn fetch_products(api_url: &str) -> Result<Vec<Product>, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/products", api_url);
    let products = client.get(&url).send().await?.json::<Vec<Product>>().await?;
    Ok(products)
}

async fn handle_health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Healthy")
}
