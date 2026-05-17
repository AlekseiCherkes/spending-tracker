use crate::dal::RecentSpending;

use super::format::parse_year_month;

const EXPORT_MONTHS: usize = 3;

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

/// Returns up to `EXPORT_MONTHS` year-months ending in `current_ym`, newest first.
/// Each entry: (year_month string, is_current).
pub(super) fn export_month_options(current_ym: &str) -> Vec<(String, bool)> {
    let Some((mut year, mut month)) = parse_year_month(current_ym) else {
        return vec![];
    };
    let mut out = Vec::with_capacity(EXPORT_MONTHS);
    out.push((format!("{:04}-{:02}", year, month), true));
    for _ in 1..EXPORT_MONTHS {
        let (y, m) = prev_month(year, month);
        year = y;
        month = m;
        out.push((format!("{:04}-{:02}", year, month), false));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub(super) fn build_csv(items: &[RecentSpending]) -> String {
    let mut out = String::from("Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes\n");
    for s in items {
        let notes = s.notes.as_deref().unwrap_or("");
        let iban = s.account_iban.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{:.2},{},{},{},{},{},{}\n",
            csv_escape(&s.created_at),
            s.amount,
            csv_escape(&s.currency_code),
            csv_escape(&s.category_name),
            csv_escape(&s.account_name),
            csv_escape(iban),
            csv_escape(&s.reporter_name),
            csv_escape(notes),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prev_month_basic() {
        assert_eq!(prev_month(2026, 5), (2026, 4));
        assert_eq!(prev_month(2026, 2), (2026, 1));
    }

    #[test]
    fn test_prev_month_wraps_year() {
        assert_eq!(prev_month(2026, 1), (2025, 12));
    }

    #[test]
    fn test_export_month_options_three_months_back() {
        let opts = export_month_options("2026-05");
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0], ("2026-05".to_string(), true));
        assert_eq!(opts[1], ("2026-04".to_string(), false));
        assert_eq!(opts[2], ("2026-03".to_string(), false));
    }

    #[test]
    fn test_export_month_options_crosses_year_boundary() {
        let opts = export_month_options("2026-01");
        assert_eq!(
            opts,
            vec![
                ("2026-01".to_string(), true),
                ("2025-12".to_string(), false),
                ("2025-11".to_string(), false),
            ]
        );
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_with_comma() {
        assert_eq!(csv_escape("a, b"), "\"a, b\"");
    }

    #[test]
    fn test_csv_escape_with_quote() {
        assert_eq!(csv_escape("she said \"hi\""), "\"she said \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_with_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_build_csv_header_and_rows() {
        let items = vec![
            RecentSpending {
                id: 1,
                amount: 15.5,
                currency_code: "EUR".into(),
                account_name: "Account A".into(),
                account_iban: Some("LT00 0000 0001".into()),
                category_name: "Продукты и хозтовары".into(),
                reporter_name: "Alice".into(),
                notes: Some("молоко, хлеб".into()),
                created_at: "2026-05-01T08:00:00".into(),
            },
            RecentSpending {
                id: 2,
                amount: 4.0,
                currency_code: "USD".into(),
                account_name: "Account C".into(),
                account_iban: None,
                category_name: "Кофе и вкусняшки".into(),
                reporter_name: "Bob".into(),
                notes: None,
                created_at: "2026-05-02T09:10:00".into(),
            },
        ];
        let csv = build_csv(&items);
        let mut lines = csv.lines();
        assert_eq!(
            lines.next(),
            Some("Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes")
        );
        assert_eq!(
            lines.next(),
            Some(
                "2026-05-01T08:00:00,15.50,EUR,Продукты и хозтовары,Account A,LT00 0000 0001,Alice,\"молоко, хлеб\""
            )
        );
        assert_eq!(
            lines.next(),
            Some("2026-05-02T09:10:00,4.00,USD,Кофе и вкусняшки,Account C,,Bob,")
        );
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn test_build_csv_empty_only_header() {
        let csv = build_csv(&[]);
        assert_eq!(
            csv,
            "Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes\n"
        );
    }
}
