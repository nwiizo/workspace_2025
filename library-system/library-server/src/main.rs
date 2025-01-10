use chrono::{DateTime, Duration, Utc};
use prost_types::Timestamp;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

// 生成されたコードを直接インポート
tonic::include_proto!("library.v1");

use library_service_server::{LibraryService, LibraryServiceServer};

// SQLXのクエリ結果を受け取るための構造体
#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    name: String,
    email: String,
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: String,
    title: String,
    author: String,
    isbn: String,
    available: bool,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)] // 未使用のフィールドがあるため警告を抑制
struct LoanRow {
    id: String,
    book_id: String,
    user_id: String,
    loan_date: chrono::DateTime<Utc>,
    due_date: chrono::DateTime<Utc>,
    return_date: Option<chrono::DateTime<Utc>>,
    status: i32,
}

pub struct LibraryServiceImpl {
    pool: Pool<Sqlite>,
}

impl LibraryServiceImpl {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    fn datetime_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}

#[tonic::async_trait]
impl LibraryService for LibraryServiceImpl {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
            .bind(&user_id)
            .bind(&req.name)
            .bind(&req.email)
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateUserResponse {
            user: Some(User {
                id: user_id,
                name: req.name,
                email: req.email,
            }),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = sqlx::query_as::<_, UserRow>("SELECT id, name, email FROM users WHERE id = ?")
            .bind(&req.user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        Ok(Response::new(GetUserResponse {
            user: Some(User {
                id: user.id,
                name: user.name,
                email: user.email,
            }),
        }))
    }

    async fn search_books(
        &self,
        request: Request<SearchBooksRequest>,
    ) -> Result<Response<SearchBooksResponse>, Status> {
        let req = request.into_inner();
        let offset = (req.page_number - 1) * req.page_size;
        let query = format!("%{}%", req.query);

        let books = sqlx::query_as::<_, BookRow>(
            r#"
            SELECT id, title, author, isbn, available 
            FROM books 
            WHERE title LIKE ? OR author LIKE ? OR isbn LIKE ?
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&query)
        .bind(&query)
        .bind(&query)
        .bind(req.page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let total_count = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COUNT(*) 
            FROM books 
            WHERE title LIKE ? OR author LIKE ? OR isbn LIKE ?
            "#,
        )
        .bind(&query)
        .bind(&query)
        .bind(&query)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let total_pages = (total_count as f64 / req.page_size as f64).ceil() as i32;

        let books = books
            .into_iter()
            .map(|b| Book {
                id: b.id,
                title: b.title,
                author: b.author,
                isbn: b.isbn,
                available: b.available,
            })
            .collect();

        Ok(Response::new(SearchBooksResponse {
            books,
            total_count,
            total_pages,
        }))
    }

    async fn create_loan(
        &self,
        request: Request<CreateLoanRequest>,
    ) -> Result<Response<CreateLoanResponse>, Status> {
        let req = request.into_inner();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let book = sqlx::query_scalar::<_, bool>("SELECT available FROM books WHERE id = ?")
            .bind(&req.book_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Book not found"))?;

        if !book {
            return Err(Status::failed_precondition("Book is not available"));
        }

        sqlx::query("UPDATE books SET available = false WHERE id = ?")
            .bind(&req.book_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let now = Utc::now();
        let due_date = now + Duration::days(14);
        let loan_id = Uuid::new_v4().to_string();

        let loan = Loan {
            id: loan_id.clone(),
            book_id: req.book_id.clone(),
            user_id: req.user_id.clone(),
            loan_date: Some(Self::datetime_to_timestamp(now)),
            due_date: Some(Self::datetime_to_timestamp(due_date)),
            return_date: None,
            status: LoanStatus::Active as i32,
        };

        sqlx::query(
            r#"
            INSERT INTO loans (id, book_id, user_id, loan_date, due_date, status)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&loan.id)
        .bind(&loan.book_id)
        .bind(&loan.user_id)
        .bind(now)
        .bind(due_date)
        .bind(LoanStatus::Active as i32)
        .execute(&mut *tx)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateLoanResponse { loan: Some(loan) }))
    }

    async fn return_book(
        &self,
        request: Request<ReturnBookRequest>,
    ) -> Result<Response<ReturnBookResponse>, Status> {
        let req = request.into_inner();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let now = Utc::now();

        let loan = sqlx::query_as::<_, LoanRow>(
            r#"
            SELECT id, book_id, user_id, loan_date, due_date, return_date, status
            FROM loans 
            WHERE id = ? AND status = ?
            "#,
        )
        .bind(&req.loan_id)
        .bind(LoanStatus::Active as i32)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("Active loan not found"))?;

        sqlx::query("UPDATE books SET available = true WHERE id = ?")
            .bind(&loan.book_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE loans 
            SET status = ?, return_date = ?
            WHERE id = ?
            "#,
        )
        .bind(LoanStatus::Returned as i32)
        .bind(now)
        .bind(&loan.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let updated_loan = Loan {
            id: loan.id,
            book_id: loan.book_id,
            user_id: loan.user_id,
            loan_date: Some(Self::datetime_to_timestamp(loan.loan_date)),
            due_date: Some(Self::datetime_to_timestamp(loan.due_date)),
            return_date: Some(Self::datetime_to_timestamp(now)),
            status: LoanStatus::Returned as i32,
        };

        tx.commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ReturnBookResponse {
            success: true,
            loan: Some(updated_loan),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:library.db".to_string());

    let service = LibraryServiceImpl::new(&database_url).await?;
    let addr = "[::1]:50051".parse()?;

    println!("LibraryService listening on {}", addr);

    Server::builder()
        .add_service(LibraryServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
