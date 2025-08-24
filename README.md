# kino-api
Runs the backend for Kino flashcard system and handles user authentication,
cards, decks, and dictionary integration.

## Setup
### Database
Load the PostgreSQL schema:
```sh
psql -d kino < database.sql
```

### Environment
Create `.env` file based on `example.env`.

Environment variables:
- `GOOGLE_CLIENT_ID`: Must be obtained from Google Cloud Console. Make sure
  the OAuth client is set up for **Web application**, and the authorized
  redirect URIs match your API or frontend setup.
- `WN_DATABASE`: The path should contain the WordNet dictionary files. Kino
  reads WordNet data directly from these files at runtime, so the directory
  must be accessible by the API server.
- `HOST`: API server host and port, e.g., `0.0.0.0:8081`.
- `DATABASE_URL`: PostgreSQL connection string, e.g.,
  `postgres://user:password@localhost/kino`
- `REDIS_URL`: Redis instance for rate limiting.
- `JWT_SECRET`: Secret key for signing JWTs. Use random password generator for
  secret.

### Build
PostgreSQL database has to be connected during build.
```sh
cargo build
```

## Run
API should run if setup above applied correctly.
```sh
cargo run
```
