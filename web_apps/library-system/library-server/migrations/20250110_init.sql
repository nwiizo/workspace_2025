-- library-server/migrations/20240110_init.sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS books (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    isbn TEXT NOT NULL UNIQUE,
    available BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE IF NOT EXISTS loans (
    id TEXT PRIMARY KEY,
    book_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    loan_date TIMESTAMP NOT NULL,
    due_date TIMESTAMP NOT NULL,
    return_date TIMESTAMP,
    status INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (book_id) REFERENCES books(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- サンプルデータ
INSERT INTO books (id, title, author, isbn, available) VALUES
    ('1', 'Rust入門', '山田太郎', '978-4-xxxx-xxxx-x', true),
    ('2', 'Advanced Rust', 'John Doe', '978-4-yyyy-yyyy-y', true),
    ('3', 'Rust and WebAssembly', 'Jane Smith', '978-4-zzzz-zzzz-z', true)
ON CONFLICT DO NOTHING;
