pub const TEST_DATA: &str =
    "
-- Insert test data for Currency
INSERT INTO Currency (name) VALUES ('EUR');
INSERT INTO Currency (name) VALUES ('USD');
INSERT INTO Currency (name) VALUES ('BYN');

-- Insert test data for User
INSERT INTO User (telegramId, telegramName, displayName) VALUES (1001, 'alex_bot', 'Alex');
INSERT INTO User (telegramId, telegramName, displayName) VALUES (1002, 'hanna_bot', 'Hanna');

-- Insert test data for Account
INSERT INTO Account (name, displayName, currencyId) VALUES
('Savings', 'Alex Savings', 1), -- EUR
('Expenses', 'Hanna Daily', 2), -- USD
('Family Fund', 'Family BYN', 3); -- BYN

-- Insert test data for ExpenseCategory
INSERT INTO ExpenseCategory (name, active, comments, sortingOrder) VALUES
('Groceries', 1, 'Food and daily groceries', 1),
('Transportation', 1, 'Bus, taxi, fuel expenses', 2),
('Entertainment', 1, 'Movies, concerts, etc.', 3),
('Utilities', 1, 'Electricity, water, internet bills', 4);

-- Insert test data for Expense
INSERT INTO Expense (accountId, categoryId, userId, timestamp, amount, comments) VALUES
(1, 1, 1001, strftime('%s', '2023-12-01 10:00:00'), 50.75, 'Bought groceries for the week'), -- Alex, EUR, Groceries
(2, 2, 1002, strftime('%s', '2023-12-02 12:30:00'), 20.00, 'Taxi ride to the office'),       -- Hanna, USD, Transportation
(3, 3, 1001, strftime('%s', '2023-12-03 18:00:00'), 30.00, 'Cinema tickets for family'),    -- Alex, BYN, Entertainment
(2, 4, 1002, strftime('%s', '2023-12-04 20:00:00'), 100.00, 'Paid electricity bill');        -- Hanna, USD, Utilities
";

