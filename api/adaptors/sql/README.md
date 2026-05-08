# SQL Adaptor

This adaptor works with [Postgres](https://www.postgresql.org/), [MySQL](https://www.mysql.com/) or [SQLite](https://sqlite.org/index.html) databases.

## Environment

To use this adaptor, make sure you have the `DATABASE_URL` environment variable set to the database url for your chosen database.

Example:

```env
DATABASE_URL="postgresql://username:password@localhost:5432/crabfit"
```

## Usage

Run `mise migrate` to apply database migrations.

Run `mise generate-entities` to generate entity definitions from the database schema.
