# Local Development

1. Create `.env` in project root:
```
TELOXIDE_TOKEN=your-dev-bot-token
RUST_LOG=debug
```

2. (Optional) Pull a snapshot of the production database:
```
./scripts/download-prod-db.sh
```
This produces `spending_tracker.db` in the project root. There is no longer any
preseeding of users/accounts/categories at startup, so without this snapshot
the local DB will only contain the empty schema.

3. Run: `cargo run`
