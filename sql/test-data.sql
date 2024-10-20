-- Вставка пользователей
INSERT INTO users (name) VALUES
('Alice'),
('Bob'),
('Charlie');

-- Вставка валют
INSERT INTO currencies (currency_code) VALUES
('USD'),
('EUR'),
('JPY');

-- Вставка банковских счетов
INSERT INTO bank_accounts (account_name, user_id, currency_id) VALUES
('Alice Savings', 1, 1),  -- Alice, USD
('Bob Checking', 2, 2),   -- Bob, EUR
('Charlie Business', 3, 3), -- Charlie, JPY
('Alice Investment', 1, 2); -- Alice, EUR

-- Вставка категорий расходов
INSERT INTO expense_categories (category_name) VALUES
('Groceries'),
('Entertainment'),
('Transport'),
('Rent');

-- Вставка транзакций
INSERT INTO transactions (user_id, account_id, category_id, transaction_timestamp, amount, notes) VALUES
(1, 1, 1, '2024-10-01 14:30:00', 50.75, 'Bought groceries at supermarket'),   -- Alice, Groceries
(2, 2, 2, '2024-10-02 18:45:00', 30.00, 'Movie night with friends'),          -- Bob, Entertainment
(3, 3, 3, '2024-10-03 08:15:00', 100.00, 'Taxi to business meeting'),         -- Charlie, Transport
(1, 4, 4, '2024-10-04 12:00:00', 1500.00, 'Monthly rent payment');            -- Alice, Rent
