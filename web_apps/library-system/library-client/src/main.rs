use tonic::{transport::Channel, Request};
use uuid::Uuid;

// 生成されたコードを直接インポート
tonic::include_proto!("library.v1");

use library_service_client::LibraryServiceClient;

async fn create_user(
    client: &mut LibraryServiceClient<Channel>,
) -> Result<User, Box<dyn std::error::Error>> {
    // UUIDを使用してユニークなメールアドレスを生成
    let unique_id = Uuid::new_v4();
    let response = client
        .create_user(Request::new(CreateUserRequest {
            name: "山田太郎".to_string(),
            email: format!("yamada_{}@example.com", unique_id),
        }))
        .await?;

    let user = response.get_ref().user.as_ref().unwrap().clone();
    println!("Created user: {:?}", user);
    Ok(user)
}

async fn search_books(
    client: &mut LibraryServiceClient<Channel>,
) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
    let response = client
        .search_books(Request::new(SearchBooksRequest {
            query: "Rust".to_string(),
            page_size: 10,
            page_number: 1,
        }))
        .await?;

    println!("\nSearch results:");
    println!("Total books found: {}", response.get_ref().total_count);
    println!("Total pages: {}", response.get_ref().total_pages);

    let books = response.get_ref().books.clone();
    for book in &books {
        println!(
            "Book: {} by {} (ISBN: {}), Available: {}",
            book.title, book.author, book.isbn, book.available
        );
    }

    Ok(books)
}

async fn create_loan(
    client: &mut LibraryServiceClient<Channel>,
    book_id: String,
    user_id: String,
) -> Result<Loan, Box<dyn std::error::Error>> {
    let response = client
        .create_loan(Request::new(CreateLoanRequest { book_id, user_id }))
        .await?;

    let loan = response.get_ref().loan.as_ref().unwrap().clone();
    println!("\nCreated loan:");
    println!("Loan ID: {}", loan.id);
    println!("Due date: {:?}", loan.due_date);

    Ok(loan)
}

async fn return_book(
    client: &mut LibraryServiceClient<Channel>,
    loan_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .return_book(Request::new(ReturnBookRequest { loan_id }))
        .await?;

    println!("\nReturned book:");
    println!("Success: {}", response.get_ref().success);
    println!("Return details: {:?}", response.get_ref().loan);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to library service...");
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let mut client = LibraryServiceClient::new(channel);

    // ユーザーを作成
    println!("\nCreating new user...");
    let user = create_user(&mut client).await?;

    // 書籍を検索
    println!("\nSearching for books...");
    let books = search_books(&mut client).await?;

    // 利用可能な本があれば借りてみる
    for book in books {
        if book.available {
            println!("\nAttempting to borrow book: {}", book.title);

            // 本を借りる
            let loan = create_loan(&mut client, book.id, user.id).await?;

            // 少し待つ
            println!("\nWaiting for 2 seconds before returning...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 本を返却
            println!("\nReturning the book...");
            return_book(&mut client, loan.id).await?;

            break;
        }
    }

    println!("\nDemo completed successfully!");
    Ok(())
}
