# pgaf

PostgreSQL extension for [Antfly](https://github.com/antflydb/antfly). Provides search functions, row sync triggers, and (planned) a custom index access method — so Antfly-powered search feels native to Postgres.

## Features

- **`antfly_search()`** — query an Antfly collection from SQL and join results back to your tables
- **`antfly_sync_trigger()`** — trigger that pushes row changes to Antfly on INSERT/UPDATE/DELETE
- **Custom Index AM** (planned) — `CREATE INDEX ... USING antfly` for planner-integrated search

## Requirements

- PostgreSQL 13–18
- Rust (edition 2024)
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.17.0

## Quick Start

```bash
# Install cargo-pgrx if you haven't already
cargo install cargo-pgrx --version 0.17.0 --locked
cargo pgrx init

# Build and install
cargo pgrx install

# Or run in a temporary dev instance
cargo pgrx run
```

Then in psql:

```sql
CREATE EXTENSION pgaf;
```

## Usage

### Search

Query an Antfly collection and join results back to a Postgres table:

```sql
SELECT t.*, s.score
FROM my_table t
JOIN antfly_search(
    'http://localhost:8080',
    'my_collection',
    'fix my computer'
) s ON t.id = s.id
ORDER BY s.score DESC;
```

The function signature:

```sql
antfly_search(base_url TEXT, collection TEXT, query TEXT, limit INT DEFAULT NULL)
RETURNS TABLE (id TEXT, score FLOAT8, data JSONB)
```

### Triggers

Automatically sync row changes to Antfly:

```sql
CREATE TRIGGER sync_to_antfly
  AFTER INSERT OR UPDATE OR DELETE ON my_table
  FOR EACH ROW
  EXECUTE FUNCTION antfly_sync_trigger(
    'http://localhost:8080',  -- Antfly server URL
    'my_collection',          -- target collection
    'id'                      -- column to use as document ID
  );
```

Every insert/update pushes the full row as JSON. Deletes remove the document from Antfly.

### Status Check

```sql
SELECT antfly_status('http://localhost:8080');
```

## Project Structure

```
src/
├── lib.rs         # Extension entry point
├── client.rs      # Antfly HTTP client
├── functions.rs   # SQL functions (antfly_search, antfly_status)
└── triggers.rs    # Trigger function (antfly_sync_trigger)
```

## Testing

```bash
cargo pgrx test
```

## License

See [LICENSE](LICENSE).
