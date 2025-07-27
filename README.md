# Setup Instructions

1. Install [SQLx CLI](https://crates.io/crates/sqlx-cli)
2. Create a `.env` file with the following contents:

```bash
DATABASE_URL=sqlite://database.db
```

3. Run `sqlx database create`
4. Run `sqlx migrate run`
5. Enter the SQLite CLI on the created database file and run `INSERT INTO human_tokens VALUES ('5040312e-c101-4fe0-abc2-acbd0db6dea3');`
6. Run the server with `cargo run -- PORT` where `PORT` is the desired port number.
7. On the page, enter "5040312e-c101-4fe0-abc2-acbd0db6dea3" in the input on the bottom left.