# Running Migrator CLI

- Generate a new migration file
  ```sh
  cargo run -- migrate generate MIGRATION_NAME
  ```
- Apply all pending migrations
  ```sh
  cargo run
  ```
  ```sh
  cargo run -- up
  ```
- Apply first 10 pending migrations
  ```sh
  cargo run -- up -n 10
  ```
- Rollback last applied migrations
  ```sh
  cargo run -- down
  ```
- Rollback last 10 applied migrations
  ```sh
  cargo run -- down -n 10
  ```
- Drop all tables from the database, then reapply all migrations
  ```sh
  cargo run -- fresh
  ```
- Rollback all applied migrations, then reapply all migrations
  ```sh
  cargo run -- refresh
  ```
- Rollback all applied migrations
  ```sh
  cargo run -- reset
  ```
- Check the status of all migrations
  ```sh
  cargo run -- status
  ```

## Database URL

All of the migration commands require you to have a `DATABASE_URL` specified in your env. [see here](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/#configure-environment) for more info.
Recommend to create a `.env` file in this directory with a `DATABASE_URL` value.
