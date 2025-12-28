//! Chapter 10: Postgres for Geospatial Data
//!
//! 書籍の10章で説明されているPostGIS機能。
//! - geometry/geography型
//! - ST_Distance, ST_DWithin（距離計算）
//! - ST_Within, ST_Contains（包含関係）
//! - ST_Transform（座標変換）
//! - GiSTインデックス

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// PostGISのデモ
///
/// 地理空間データの格納と検索機能を検証。
/// フロリダ州の観光スポットをシミュレートして検証する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "PostGIS検証（Chapter 10: Geospatial Data）".bold());
    println!("{}", "-".repeat(50));

    // PostGIS拡張を有効化
    println!("{}", "【PostGIS拡張の有効化】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS postgis", &[])
        .await?;

    println!("  {} PostGIS拡張を有効化しました", "✓".green());

    // バージョン確認
    let version = client
        .query_one("SELECT PostGIS_Version() AS version", &[])
        .await?;
    let ver: String = version.get("version");
    println!("  PostGISバージョン: {}", ver.cyan());

    // floridaスキーマを作成
    client
        .execute("CREATE SCHEMA IF NOT EXISTS florida", &[])
        .await?;

    // 既存のテーブルを削除（デモ用）
    client
        .execute("DROP TABLE IF EXISTS florida.points_of_interest CASCADE", &[])
        .await?;
    client
        .execute("DROP TABLE IF EXISTS florida.cities CASCADE", &[])
        .await?;

    // geometry型とgeography型の説明
    println!("\n{}", "【geometry型とgeography型】".yellow());
    println!("{}", "  geometry: 平面座標系（高速だが大距離で誤差）".dimmed());
    println!(
        "{}",
        "  geography: 球面座標系（正確だが計算コスト高）".dimmed()
    );

    // 都市テーブルを作成（ポリゴン）
    println!("\n{}", "【都市テーブルの作成（Polygon）】".yellow());

    client
        .execute(
            "CREATE TABLE florida.cities (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                place TEXT,
                -- SRID 4326: WGS 84（GPS座標系、緯度経度）
                boundaries geometry(Polygon, 4326)
            )",
            &[],
        )
        .await?;

    println!("  {} florida.cities テーブルを作成（geometry型）", "✓".green());

    // 都市データを挿入（簡略化した境界）
    client
        .execute(
            r#"
            INSERT INTO florida.cities (name, place, boundaries) VALUES
            ('Tampa', 'city', ST_GeomFromText(
                'POLYGON((-82.55 27.85, -82.35 27.85, -82.35 28.05, -82.55 28.05, -82.55 27.85))',
                4326
            )),
            ('Miami', 'city', ST_GeomFromText(
                'POLYGON((-80.35 25.7, -80.1 25.7, -80.1 25.9, -80.35 25.9, -80.35 25.7))',
                4326
            )),
            ('Orlando', 'city', ST_GeomFromText(
                'POLYGON((-81.5 28.35, -81.25 28.35, -81.25 28.6, -81.5 28.6, -81.5 28.35))',
                4326
            ))
            "#,
            &[],
        )
        .await?;

    println!("  {} 3都市のデータを挿入", "✓".green());

    // POIテーブルを作成（ポイント）
    println!("\n{}", "【観光スポットテーブルの作成（Point）】".yellow());

    client
        .execute(
            "CREATE TABLE florida.points_of_interest (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                tourism TEXT,
                amenity TEXT,
                -- SRID 4326: WGS 84
                location geometry(Point, 4326)
            )",
            &[],
        )
        .await?;

    println!(
        "  {} florida.points_of_interest テーブルを作成",
        "✓".green()
    );

    // POIデータを挿入
    client
        .execute(
            r#"
            INSERT INTO florida.points_of_interest (name, tourism, amenity, location) VALUES
            -- Tampa エリア
            ('Roost Apartment Hotel', 'hotel', NULL,
             ST_SetSRID(ST_MakePoint(-82.4584, 27.9478), 4326)),
            ('Wagamama Tampa', NULL, 'restaurant',
             ST_SetSRID(ST_MakePoint(-82.4590, 27.9472), 4326)),
            ('Boulon Brasserie', NULL, 'restaurant',
             ST_SetSRID(ST_MakePoint(-82.4578, 27.9465), 4326)),
            ('Busch Gardens Tampa Bay', 'theme_park', NULL,
             ST_SetSRID(ST_MakePoint(-82.4209, 28.0378), 4326)),

            -- Orlando エリア（Walt Disney World）
            ('Walt Disney World', 'theme_park', NULL,
             ST_SetSRID(ST_MakePoint(-81.5639, 28.3852), 4326)),
            ('Magic Kingdom', 'theme_park', NULL,
             ST_SetSRID(ST_MakePoint(-81.5812, 28.4177), 4326)),
            ('EPCOT', 'theme_park', NULL,
             ST_SetSRID(ST_MakePoint(-81.5494, 28.3747), 4326)),
            ('Universal Studios Florida', 'theme_park', NULL,
             ST_SetSRID(ST_MakePoint(-81.4684, 28.4755), 4326)),

            -- Miami エリア
            ('Brickell City Centre', NULL, 'shopping',
             ST_SetSRID(ST_MakePoint(-80.1918, 25.7650), 4326)),
            ('Pérez Art Museum Miami', 'museum', NULL,
             ST_SetSRID(ST_MakePoint(-80.1867, 25.7857), 4326)),
            ('South Beach', 'attraction', NULL,
             ST_SetSRID(ST_MakePoint(-80.1280, 25.7826), 4326)),
            ('Miami Zoo', 'zoo', NULL,
             ST_SetSRID(ST_MakePoint(-80.4012, 25.6110), 4326))
            "#,
            &[],
        )
        .await?;

    let count = client
        .query_one(
            "SELECT COUNT(*) FROM florida.points_of_interest",
            &[],
        )
        .await?;
    let cnt: i64 = count.get(0);
    println!("  {} {} 件の観光スポットを挿入", "✓".green(), cnt);

    // ST_AsText: 座標の表示
    println!("\n{}", "【座標の取得（ST_AsText）】".yellow());

    let tampa = client
        .query_one(
            "SELECT name, ST_AsText(location) AS wkt
             FROM florida.points_of_interest
             WHERE name = 'Roost Apartment Hotel'",
            &[],
        )
        .await?;
    let name: String = tampa.get("name");
    let wkt: String = tampa.get("wkt");
    println!("  {}: {}", name.cyan(), wkt);

    // ST_Distance: 距離計算
    println!("\n{}", "【距離計算（ST_Distance）】".yellow());
    println!(
        "{}",
        "  geography型にキャストして正確な距離（メートル）を計算".dimmed()
    );

    let distances = client
        .query(
            r#"
            WITH hotel AS (
                SELECT location FROM florida.points_of_interest
                WHERE name = 'Roost Apartment Hotel'
            )
            SELECT
                p.name,
                p.amenity,
                round(ST_Distance(
                    p.location::geography,
                    h.location::geography
                ))::INT AS distance_meters
            FROM florida.points_of_interest p, hotel h
            WHERE p.amenity = 'restaurant'
            ORDER BY distance_meters
            "#,
            &[],
        )
        .await?;

    println!("  Roost Apartment Hotelからレストランまでの距離:");
    for row in distances {
        let name: String = row.get("name");
        let dist: i32 = row.get("distance_meters");
        println!("    {} - {} メートル", name.cyan(), dist);
    }

    // ST_DWithin: 範囲検索
    println!("\n{}", "【範囲検索（ST_DWithin）】".yellow());
    println!("{}", "  指定距離内の地点を効率的に検索".dimmed());

    let nearby = client
        .query(
            r#"
            WITH hotel AS (
                SELECT location FROM florida.points_of_interest
                WHERE name = 'Roost Apartment Hotel'
            )
            SELECT p.name, p.tourism, p.amenity
            FROM florida.points_of_interest p, hotel h
            WHERE ST_DWithin(
                p.location::geography,
                h.location::geography,
                1000  -- 1000メートル以内
            )
            AND p.name != 'Roost Apartment Hotel'
            "#,
            &[],
        )
        .await?;

    println!("  ホテルから1km以内の施設:");
    for row in nearby {
        let name: String = row.get("name");
        let tourism: Option<String> = row.get("tourism");
        let amenity: Option<String> = row.get("amenity");
        let category = tourism.or(amenity).unwrap_or_default();
        println!("    {} ({})", name.cyan(), category);
    }

    // ST_Within: 包含関係
    println!("\n{}", "【包含関係（ST_Within）】".yellow());
    println!("{}", "  ポイントがポリゴン内にあるかチェック".dimmed());

    let pois_in_cities = client
        .query(
            r#"
            SELECT c.name AS city, p.name AS poi, p.tourism
            FROM florida.points_of_interest p
            JOIN florida.cities c ON ST_Within(p.location, c.boundaries)
            WHERE p.tourism IS NOT NULL
            ORDER BY c.name, p.name
            "#,
            &[],
        )
        .await?;

    println!("  各都市内の観光スポット:");
    let mut current_city = String::new();
    for row in pois_in_cities {
        let city: String = row.get("city");
        let poi: String = row.get("poi");
        let tourism: String = row.get("tourism");
        if city != current_city {
            println!("  {}:", city.yellow());
            current_city = city;
        }
        println!("    - {} ({})", poi.cyan(), tourism);
    }

    // ST_Transform: 座標変換
    println!("\n{}", "【座標変換（ST_Transform）】".yellow());
    println!(
        "{}",
        "  WGS 84（SRID 4326）からWeb Mercator（SRID 3857）へ".dimmed()
    );

    let transformed = client
        .query_one(
            r#"
            SELECT
                name,
                ST_X(location) AS lon_4326,
                ST_Y(location) AS lat_4326,
                ST_X(ST_Transform(location, 3857)) AS x_3857,
                ST_Y(ST_Transform(location, 3857)) AS y_3857
            FROM florida.points_of_interest
            WHERE name = 'Walt Disney World'
            "#,
            &[],
        )
        .await?;

    let name: String = transformed.get("name");
    let lon: f64 = transformed.get("lon_4326");
    let lat: f64 = transformed.get("lat_4326");
    let x: f64 = transformed.get("x_3857");
    let y: f64 = transformed.get("y_3857");
    println!("  {}:", name.cyan());
    println!("    WGS 84 (SRID 4326): 経度 {:.4}, 緯度 {:.4}", lon, lat);
    println!("    Web Mercator (SRID 3857): X {:.2}, Y {:.2}", x, y);

    // GiSTインデックス
    println!("\n{}", "【GiSTインデックス】".yellow());
    println!(
        "{}",
        "  空間検索を高速化するための専用インデックス".dimmed()
    );

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_poi_location_gist
             ON florida.points_of_interest
             USING GIST (location)",
            &[],
        )
        .await?;

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_cities_boundaries_gist
             ON florida.cities
             USING GIST (boundaries)",
            &[],
        )
        .await?;

    println!("  {} GiSTインデックスを作成", "✓".green());

    // 実行計画の確認
    let explain = client
        .query(
            r#"
            EXPLAIN (COSTS OFF)
            SELECT p.name
            FROM florida.points_of_interest p
            JOIN florida.cities c ON ST_Within(p.location, c.boundaries)
            WHERE c.name = 'Miami'
            "#,
            &[],
        )
        .await?;

    println!("  実行計画:");
    for row in explain.iter().take(6) {
        let plan: String = row.get(0);
        if plan.contains("Index") {
            println!("    {}", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // 都市間の距離
    println!("\n{}", "【都市間の距離】".yellow());

    let city_distances = client
        .query(
            r#"
            SELECT
                c1.name AS from_city,
                c2.name AS to_city,
                round(ST_Distance(
                    ST_Centroid(c1.boundaries)::geography,
                    ST_Centroid(c2.boundaries)::geography
                ) / 1000)::INT AS distance_km
            FROM florida.cities c1
            CROSS JOIN florida.cities c2
            WHERE c1.name < c2.name
            ORDER BY distance_km
            "#,
            &[],
        )
        .await?;

    for row in city_distances {
        let from: String = row.get("from_city");
        let to: String = row.get("to_city");
        let dist: i32 = row.get("distance_km");
        println!("  {} → {}: {} km", from.cyan(), to.cyan(), dist);
    }

    // OpenStreetMap URLの生成
    println!("\n{}", "【OpenStreetMap URLの生成】".yellow());

    let osm_url = client
        .query_one(
            r#"
            SELECT
                'https://www.openstreetmap.org/?mlat=' ||
                ST_Y(location) || '&mlon=' || ST_X(location) ||
                '&zoom=15' AS url
            FROM florida.points_of_interest
            WHERE name = 'Magic Kingdom'
            "#,
            &[],
        )
        .await?;
    let url: String = osm_url.get("url");
    println!("  Magic Kingdom: {}", url.cyan());

    // 使い分けの説明
    println!("\n{}", "【PostGISを使う基準】".yellow());
    println!("  {}:", "✓ 適切なケース".green());
    println!("    - 店舗/施設の位置検索");
    println!("    - 配達範囲の計算");
    println!("    - 地理的境界の管理");
    println!("    - ルート計算（pgRoutingと併用）");

    println!("  {}:", "✗ 専用GISを検討すべきケース".red());
    println!("    - 大規模な地図タイル配信");
    println!("    - リアルタイム車両追跡（大量更新）");
    println!("    - 高度な地図レンダリング");

    // PostGIS関数のサマリー
    println!("\n{}", "【主要なPostGIS関数】".yellow());
    println!("  {}:", "作成・変換".cyan());
    println!("    ST_MakePoint(x, y) - ポイント作成");
    println!("    ST_GeomFromText(wkt, srid) - WKTからジオメトリ作成");
    println!("    ST_Transform(geom, srid) - 座標系変換");

    println!("  {}:", "測定".cyan());
    println!("    ST_Distance(g1, g2) - 距離計算");
    println!("    ST_Area(geom) - 面積計算");
    println!("    ST_Length(geom) - 線の長さ");

    println!("  {}:", "関係".cyan());
    println!("    ST_Within(g1, g2) - g1がg2内にあるか");
    println!("    ST_Contains(g1, g2) - g1がg2を含むか");
    println!("    ST_Intersects(g1, g2) - 交差するか");
    println!("    ST_DWithin(g1, g2, dist) - 距離内か");

    println!(
        "\n{}",
        "💡 ポイント: 既にPostgresを使っているなら、PostGISで地理空間機能を追加可能。".dimmed()
    );
    println!(
        "{}",
        "   MongoDBのGeoJSONやElasticsearchの前に、まずPostGISで試す。".dimmed()
    );

    Ok(())
}
