//! Database utilities using SQLite

use crate::error::AppError;
use crate::models::{Order, Payment, User};
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new in-memory database
    pub fn new_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Create a database from file
    pub fn new_from_file(path: &str) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user TEXT NOT NULL,
                product TEXT NOT NULL,
                quantity INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS payments (
                id TEXT PRIMARY KEY,
                amount REAL NOT NULL,
                currency TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Seed sample data for orders
    pub fn seed_orders(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        // Check if already seeded
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(());
        }

        conn.execute(
            "INSERT INTO orders (user, product, quantity) VALUES (?1, ?2, ?3)",
            params!["alice", "Widget A", 5],
        )?;
        conn.execute(
            "INSERT INTO orders (user, product, quantity) VALUES (?1, ?2, ?3)",
            params!["bob", "Widget B", 3],
        )?;
        conn.execute(
            "INSERT INTO orders (user, product, quantity) VALUES (?1, ?2, ?3)",
            params!["alice", "Widget C", 10],
        )?;

        Ok(())
    }

    /// Get order by ID (no authorization check - vulnerable)
    pub fn get_order_by_id(&self, id: i64) -> Result<Option<Order>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, user, product, quantity FROM orders WHERE id = ?1")?;

        let order = stmt
            .query_row(params![id], |row| {
                Ok(Order {
                    id: row.get(0)?,
                    user: row.get(1)?,
                    product: row.get(2)?,
                    quantity: row.get(3)?,
                })
            })
            .ok();

        Ok(order)
    }

    /// Get order by ID with user check (secure)
    pub fn get_order_by_id_for_user(&self, id: i64, user: &str) -> Result<Option<Order>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user, product, quantity FROM orders WHERE id = ?1 AND user = ?2",
        )?;

        let order = stmt
            .query_row(params![id, user], |row| {
                Ok(Order {
                    id: row.get(0)?,
                    user: row.get(1)?,
                    product: row.get(2)?,
                    quantity: row.get(3)?,
                })
            })
            .ok();

        Ok(order)
    }

    /// Get all orders for a user
    pub fn get_orders_for_user(&self, user: &str) -> Result<Vec<Order>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, user, product, quantity FROM orders WHERE user = ?1")?;

        let orders = stmt
            .query_map(params![user], |row| {
                Ok(Order {
                    id: row.get(0)?,
                    user: row.get(1)?,
                    product: row.get(2)?,
                    quantity: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(orders)
    }

    /// Create a new order
    pub fn create_order(
        &self,
        user: &str,
        product: &str,
        quantity: i32,
    ) -> Result<Order, AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO orders (user, product, quantity) VALUES (?1, ?2, ?3)",
            params![user, product, quantity],
        )?;
        let id = conn.last_insert_rowid();

        Ok(Order {
            id,
            user: user.to_string(),
            product: product.to_string(),
            quantity,
        })
    }

    /// Create a payment (safe version)
    pub fn create_payment(&self, payment: &Payment) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO payments (id, amount, currency, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![payment.id, payment.amount, payment.currency, payment.status, payment.created_at],
        )?;
        Ok(())
    }

    /// Get payment by ID
    pub fn get_payment_by_id(&self, id: &str) -> Result<Option<Payment>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, currency, status, created_at FROM payments WHERE id = ?1",
        )?;

        let payment = stmt
            .query_row(params![id], |row| {
                Ok(Payment {
                    id: row.get(0)?,
                    amount: row.get(1)?,
                    currency: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .ok();

        Ok(payment)
    }

    /// Create a user
    pub fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User, AppError> {
        let conn = self.conn.lock().unwrap();
        let created_at = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO users (email, password_hash, role, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![email, password_hash, role, created_at],
        )?;
        let id = conn.last_insert_rowid();

        Ok(User {
            id,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: role.to_string(),
            created_at,
        })
    }

    /// Get user by email
    pub fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, role, created_at FROM users WHERE email = ?1",
        )?;

        let user = stmt
            .query_row(params![email], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .ok();

        Ok(user)
    }

    /// Seed sample users
    pub fn seed_users(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        // Check if already seeded
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(());
        }

        drop(conn); // Release lock before calling create_user

        // Using argon2 password hashing
        use argon2::{
            Argon2,
            password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
        };

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        let admin_hash = argon2
            .hash_password(b"admin123", &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        let user_hash = argon2
            .hash_password(b"user123", &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        self.create_user("admin@example.com", &admin_hash, "admin")?;
        self.create_user("user@example.com", &user_hash, "user")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_order() {
        let db = Database::new_in_memory().unwrap();
        let order = db.create_order("alice", "Test Product", 5).unwrap();

        assert_eq!(order.user, "alice");
        assert_eq!(order.product, "Test Product");
        assert_eq!(order.quantity, 5);

        let fetched = db.get_order_by_id(order.id).unwrap().unwrap();
        assert_eq!(fetched.id, order.id);
    }

    #[test]
    fn test_order_authorization() {
        let db = Database::new_in_memory().unwrap();
        let order = db.create_order("alice", "Test Product", 5).unwrap();

        // Alice can access her order
        let result = db.get_order_by_id_for_user(order.id, "alice").unwrap();
        assert!(result.is_some());

        // Bob cannot access Alice's order
        let result = db.get_order_by_id_for_user(order.id, "bob").unwrap();
        assert!(result.is_none());
    }
}
