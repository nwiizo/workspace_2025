use reqwest::Error;
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
struct Post {
    id: i32,
    title: String,
    body: String,
}

#[derive(Debug)]
struct UserProfile {
    user: User,
    posts: Vec<Post>,
    comments_count: usize,
}

async fn fetch_user(user_id: i32) -> Result<User, Error> {
    let url = format!("https://jsonplaceholder.typicode.com/users/{}", user_id);
    reqwest::get(&url).await?.json::<User>().await
}

async fn fetch_user_posts(user_id: i32) -> Result<Vec<Post>, Error> {
    let url = format!(
        "https://jsonplaceholder.typicode.com/posts?userId={}",
        user_id
    );
    reqwest::get(&url).await?.json::<Vec<Post>>().await
}

async fn fetch_comments_count(user_id: i32) -> Result<usize, Error> {
    let url = format!(
        "https://jsonplaceholder.typicode.com/comments?postId={}",
        user_id
    );
    let comments = reqwest::get(&url)
        .await?
        .json::<Vec<serde_json::Value>>()
        .await?;
    Ok(comments.len())
}

async fn get_user_profile(user_id: i32) -> Result<UserProfile, Error> {
    // 3つのAPIを並行して呼び出す
    let (user_result, posts_result, comments_result) = tokio::join!(
        fetch_user(user_id),
        fetch_user_posts(user_id),
        fetch_comments_count(user_id),
    );

    Ok(UserProfile {
        user: user_result?,
        posts: posts_result?,
        comments_count: comments_result?,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let start = Instant::now();

    let profile = get_user_profile(1).await?;

    let duration = start.elapsed();

    println!("ユーザー: {}", profile.user.name);
    println!("投稿数: {}", profile.posts.len());
    println!("コメント数: {}", profile.comments_count);
    println!("\n処理時間: {} ms", duration.as_millis());

    Ok(())
}
