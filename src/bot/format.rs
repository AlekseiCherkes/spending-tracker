use crate::dal::{Account, Category, Currency, RecentSpending, User};

use super::keyboards;

fn russian_month_name(month: u32) -> &'static str {
    match month {
        1 => "Январь",
        2 => "Февраль",
        3 => "Март",
        4 => "Апрель",
        5 => "Май",
        6 => "Июнь",
        7 => "Июль",
        8 => "Август",
        9 => "Сентябрь",
        10 => "Октябрь",
        11 => "Ноябрь",
        12 => "Декабрь",
        _ => "",
    }
}

pub(super) fn parse_year_month(year_month: &str) -> Option<(i32, u32)> {
    let (y, m) = year_month.split_once('-')?;
    let year: i32 = y.parse().ok()?;
    let month: u32 = m.parse().ok().filter(|m| (1..=12).contains(m))?;
    Some((year, month))
}

pub(super) fn format_month_label(year_month: &str, is_current: bool) -> String {
    match parse_year_month(year_month) {
        Some((year, month)) => {
            let name = russian_month_name(month);
            if is_current {
                format!("{} {} (текущий)", name, year)
            } else {
                format!("{} {}", name, year)
            }
        }
        None => year_month.to_string(),
    }
}

pub(super) fn format_short_datetime(iso: &str) -> String {
    let (date_part, time_part) = iso.split_once('T').unwrap_or((iso, ""));
    let hm: String = time_part.split(':').take(2).collect::<Vec<_>>().join(":");
    if hm.is_empty() {
        date_part.to_string()
    } else {
        format!("{} {}", date_part, hm)
    }
}

pub(super) fn format_recent_spendings(items: &[RecentSpending]) -> String {
    if items.is_empty() {
        return "🧾 Транзакций пока нет".to_string();
    }
    let mut out = String::from("🧾 Последние транзакции (нажмите номер, чтобы изменить)\n\n");
    for (i, s) in items.iter().enumerate() {
        let when = format_short_datetime(&s.created_at);
        let cat = keyboards::format_category(&s.category_name);
        out.push_str(&format!(
            "{}. {} — {:.2} {} — {} — {}\n",
            i + 1,
            when,
            s.amount,
            s.currency_code,
            cat,
            s.reporter_name
        ));
        if let Some(note) = s.notes.as_deref().filter(|n| !n.is_empty()) {
            out.push_str(&format!("   📝 {}\n", note));
        }
    }
    out.trim_end().to_string()
}

pub(super) fn format_users(users: &[User]) -> String {
    let mut out = String::from("👥 Пользователи\n\n");
    for u in users {
        if u.is_admin {
            out.push_str(&format!("• {} 👑 admin\n", u.name));
        } else {
            out.push_str(&format!("• {}\n", u.name));
        }
    }
    out.trim_end().to_string()
}

pub(super) fn format_currencies(currencies: &[Currency]) -> String {
    let mut out = String::from("💱 Валюты\n\n");
    for c in currencies {
        out.push_str(&format!("• {}\n", c.currency_code));
    }
    out.trim_end().to_string()
}

pub(super) fn format_categories(categories: &[Category]) -> String {
    let mut out = String::from("📋 Категории\n\n");
    for (i, c) in categories.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}\n",
            i + 1,
            keyboards::format_category(&c.name)
        ));
    }
    out.trim_end().to_string()
}

pub(super) fn format_accounts(
    accounts: &[Account],
    users: &[User],
    currencies: &[Currency],
) -> String {
    let code_of = |id: i64| -> &str {
        currencies
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.currency_code.as_str())
            .unwrap_or("?")
    };

    let mut out = String::from("💼 Счета\n\n");

    for u in users {
        let owned: Vec<&Account> = accounts
            .iter()
            .filter(|a| a.owner_id == Some(u.id))
            .collect();
        if owned.is_empty() {
            continue;
        }
        out.push_str(&format!("👤 {}\n", u.name));
        for a in owned {
            let default_mark = if u.default_account_id == Some(a.id) {
                " ⭐ по умолчанию"
            } else {
                ""
            };
            out.push_str(&format!(
                "• {} — {}{}\n",
                a.name,
                code_of(a.currency_id),
                default_mark
            ));
            if let Some(iban) = &a.iban {
                out.push_str(&format!("  IBAN: {}\n", iban));
            }
        }
        out.push('\n');
    }

    let unassigned: Vec<&Account> = accounts.iter().filter(|a| a.owner_id.is_none()).collect();
    if !unassigned.is_empty() {
        out.push_str("❓ Без владельца\n");
        for a in unassigned {
            out.push_str(&format!("• {} — {}\n", a.name, code_of(a.currency_id)));
            if let Some(iban) = &a.iban {
                out.push_str(&format!("  IBAN: {}\n", iban));
            }
        }
    }

    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_users_marks_admin() {
        let users = vec![
            User {
                id: 1,
                name: "Alice".into(),
                telegram_id: 1,
                is_admin: true,
                default_account_id: None,
            },
            User {
                id: 2,
                name: "Bob".into(),
                telegram_id: 2,
                is_admin: false,
                default_account_id: None,
            },
        ];
        let out = format_users(&users);
        assert!(out.contains("Alice 👑 admin"));
        assert!(out.contains("• Bob"));
        assert!(!out.contains("Bob 👑"));
    }

    #[test]
    fn test_format_currencies_lists_codes() {
        let currencies = vec![
            Currency {
                id: 1,
                currency_code: "EUR".into(),
            },
            Currency {
                id: 2,
                currency_code: "USD".into(),
            },
        ];
        let out = format_currencies(&currencies);
        assert!(out.contains("• EUR"));
        assert!(out.contains("• USD"));
    }

    #[test]
    fn test_format_categories_numbered_in_order() {
        let categories = vec![
            Category {
                id: 1,
                name: "Продукты и хозтовары".into(),
                sort_order: 0,
            },
            Category {
                id: 2,
                name: "Другое".into(),
                sort_order: 1,
            },
        ];
        let out = format_categories(&categories);
        let p_idx = out.find("Продукты").unwrap();
        let o_idx = out.find("Другое").unwrap();
        assert!(p_idx < o_idx);
        assert!(out.contains("1. 🛒 Продукты и хозтовары"));
        assert!(out.contains("2. 📦 Другое"));
    }

    #[test]
    fn test_format_accounts_groups_and_marks_default() {
        let users = vec![
            User {
                id: 1,
                name: "Alice".into(),
                telegram_id: 1,
                is_admin: true,
                default_account_id: Some(10),
            },
            User {
                id: 2,
                name: "Bob".into(),
                telegram_id: 2,
                is_admin: false,
                default_account_id: None,
            },
        ];
        let currencies = vec![Currency {
            id: 1,
            currency_code: "EUR".into(),
        }];
        let accounts = vec![
            Account {
                id: 10,
                name: "Account A".into(),
                currency_id: 1,
                owner_id: Some(1),
                iban: Some("LT00".into()),
            },
            Account {
                id: 11,
                name: "Account B".into(),
                currency_id: 1,
                owner_id: Some(1),
                iban: None,
            },
            Account {
                id: 12,
                name: "Account C".into(),
                currency_id: 1,
                owner_id: Some(2),
                iban: None,
            },
        ];
        let out = format_accounts(&accounts, &users, &currencies);
        assert!(out.contains("👤 Alice"));
        assert!(out.contains("👤 Bob"));
        assert!(out.contains("Account A — EUR ⭐ по умолчанию"));
        assert!(out.contains("Account B — EUR\n"));
        assert!(!out.contains("Account B — EUR ⭐"));
        assert!(out.contains("IBAN: LT00"));
        let alice_idx = out.find("Alice").unwrap();
        let bob_idx = out.find("Bob").unwrap();
        assert!(alice_idx < bob_idx);
    }

    #[test]
    fn test_format_accounts_shows_unassigned() {
        let users = vec![];
        let currencies = vec![Currency {
            id: 1,
            currency_code: "EUR".into(),
        }];
        let accounts = vec![Account {
            id: 1,
            name: "Orphan".into(),
            currency_id: 1,
            owner_id: None,
            iban: None,
        }];
        let out = format_accounts(&accounts, &users, &currencies);
        assert!(out.contains("❓ Без владельца"));
        assert!(out.contains("Orphan — EUR"));
    }

    #[test]
    fn test_format_short_datetime_iso() {
        assert_eq!(
            format_short_datetime("2026-05-16T14:30:25"),
            "2026-05-16 14:30"
        );
    }

    #[test]
    fn test_format_short_datetime_no_time() {
        assert_eq!(format_short_datetime("2026-05-16"), "2026-05-16");
    }

    #[test]
    fn test_format_recent_spendings_empty() {
        assert_eq!(format_recent_spendings(&[]), "🧾 Транзакций пока нет");
    }

    #[test]
    fn test_format_recent_spendings_with_and_without_notes() {
        let items = vec![
            RecentSpending {
                id: 2,
                amount: 15.5,
                currency_code: "EUR".into(),
                account_name: "Account A".into(),
                account_iban: None,
                category_name: "Продукты и хозтовары".into(),
                reporter_name: "Alice".into(),
                notes: Some("молоко".into()),
                created_at: "2026-05-16T14:30:25".into(),
            },
            RecentSpending {
                id: 1,
                amount: 4.0,
                currency_code: "EUR".into(),
                account_name: "Account A".into(),
                account_iban: None,
                category_name: "Кофе и вкусняшки".into(),
                reporter_name: "Bob".into(),
                notes: None,
                created_at: "2026-05-15T09:10:00".into(),
            },
        ];
        let out = format_recent_spendings(&items);
        assert!(out.starts_with("🧾 Последние транзакции"));
        assert!(out.contains("1. 2026-05-16 14:30 — 15.50 EUR — 🛒 Продукты и хозтовары — Alice"));
        assert!(out.contains("   📝 молоко"));
        assert!(out.contains("2. 2026-05-15 09:10 — 4.00 EUR — ☕ Кофе и вкусняшки — Bob"));
        let alice_idx = out.find("Alice").unwrap();
        let bob_idx = out.find("Bob").unwrap();
        assert!(alice_idx < bob_idx);
        assert_eq!(out.matches("📝").count(), 1);
    }

    #[test]
    fn test_parse_year_month_valid() {
        assert_eq!(parse_year_month("2026-05"), Some((2026, 5)));
        assert_eq!(parse_year_month("2026-12"), Some((2026, 12)));
    }

    #[test]
    fn test_parse_year_month_invalid() {
        assert!(parse_year_month("2026-00").is_none());
        assert!(parse_year_month("2026-13").is_none());
        assert!(parse_year_month("2026").is_none());
        assert!(parse_year_month("abc-de").is_none());
    }

    #[test]
    fn test_format_month_label() {
        assert_eq!(format_month_label("2026-05", true), "Май 2026 (текущий)");
        assert_eq!(format_month_label("2026-04", false), "Апрель 2026");
        assert_eq!(format_month_label("2025-12", false), "Декабрь 2025");
        assert_eq!(format_month_label("bogus", false), "bogus");
    }
}
