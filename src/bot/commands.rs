#[derive(Debug, PartialEq)]
pub(super) enum Command {
    Start,
    Accounts,
    Categories,
    Currencies,
    Users,
    DefaultAccount,
    Recent,
    Export,
}

pub(super) fn parse_command(text: &str) -> Option<Command> {
    let first = text.split_whitespace().next()?;
    let name = first.split('@').next()?;
    match name {
        "/start" => Some(Command::Start),
        "/accounts" => Some(Command::Accounts),
        "/categories" => Some(Command::Categories),
        "/currencies" => Some(Command::Currencies),
        "/users" => Some(Command::Users),
        "/default_account" => Some(Command::DefaultAccount),
        "/recent" => Some(Command::Recent),
        "/export" => Some(Command::Export),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_known_commands() {
        assert_eq!(parse_command("/start"), Some(Command::Start));
        assert_eq!(parse_command("/accounts"), Some(Command::Accounts));
        assert_eq!(parse_command("/categories"), Some(Command::Categories));
        assert_eq!(parse_command("/currencies"), Some(Command::Currencies));
        assert_eq!(parse_command("/users"), Some(Command::Users));
        assert_eq!(
            parse_command("/default_account"),
            Some(Command::DefaultAccount)
        );
        assert_eq!(parse_command("/recent"), Some(Command::Recent));
        assert_eq!(parse_command("/export"), Some(Command::Export));
    }

    #[test]
    fn test_parse_command_with_bot_suffix() {
        assert_eq!(parse_command("/users@MyBot"), Some(Command::Users));
    }

    #[test]
    fn test_parse_command_ignores_trailing_args() {
        assert_eq!(parse_command("/accounts please"), Some(Command::Accounts));
    }

    #[test]
    fn test_parse_command_unknown() {
        assert_eq!(parse_command("/wat"), None);
        assert_eq!(parse_command("hello"), None);
        assert_eq!(parse_command(""), None);
    }
}
