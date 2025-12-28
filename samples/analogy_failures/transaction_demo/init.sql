-- トランザクションデモ用のテーブル作成

CREATE TABLE accounts (
    user_id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    CHECK (balance >= 0)
);

CREATE TABLE transfer_notifications (
    id SERIAL PRIMARY KEY,
    from_user VARCHAR(50) NOT NULL,
    to_user VARCHAR(50) NOT NULL,
    amount BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 初期データ
INSERT INTO accounts (user_id, name, balance) VALUES
    ('alice', 'Alice', 100000),
    ('bob', 'Bob', 50000);
