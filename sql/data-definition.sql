CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE currencies (
    currency_id SERIAL PRIMARY KEY,
    currency_code VARCHAR(3) NOT NULL --E.g 'USD', 'EUR', etc
);

CREATE TABLE bank_accounts (
    account_id SERIAL PRIMARY KEY,
    account_name VARCHAR(255) NOT NULL,
    user_id INT NOT NULL,
    currency_id INT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (currency_id) REFERENCES currencies (currency_id)
);

CREATE TABLE expense_categories (
    category_id SERIAL PRIMARY KEY,
    category_name VARCHAR(255) NOT NULL
);

CREATE TABLE transactions (
    transaction_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    account_id INT NOT NULL,
    category_id INT NOT NULL,
    transaction_timestamp TIMESTAMP NOT NULL,
    amount NUMERIC(10, 2) NOT NULL,  -- Сумма транзакции (например, 100.00)
    notes TEXT,  -- Произвольный текст примечаний
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES bank_accounts (account_id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES expense_categories (category_id) ON DELETE CASCADE
);
