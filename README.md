# pgaf

PostgreSQL extension for [Antfly](https://github.com/antflydb/antfly). Provides a custom index access method, search functions, and row sync triggers — so Antfly-powered search feels native to Postgres.

## Features

- **Custom Index AM** — `CREATE INDEX ... USING antfly` for planner-integrated search
- **`antfly_search()`** — query an Antfly collection from SQL and join results back to your tables
- **`antfly_sync_trigger()`** — trigger that pushes row changes to Antfly on INSERT/UPDATE/DELETE

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

### Index Access Method

Create an Antfly-backed index on a text column:

```sql
CREATE INDEX idx_content ON docs USING antfly (content)
  WITH (url = 'http://localhost:8080', collection = 'my_docs');
```

Query naturally — the planner uses the Antfly index:

```sql
SELECT * FROM docs WHERE content @@@ 'fix my computer';
```

The `@@@` operator delegates search to Antfly. On `CREATE INDEX`, all existing rows are pushed to Antfly. Subsequent inserts are synced automatically via the index AM.

**WITH options:**

| Option | Default | Description |
|--------|---------|-------------|
| `url` | `http://localhost:8080` | Antfly server URL |
| `collection` | table name | Target Antfly collection |

### Search Function

For cases where you need scores or want to join search results explicitly:

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

```sql
antfly_search(base_url TEXT, collection TEXT, query TEXT, limit INT DEFAULT NULL)
RETURNS TABLE (id TEXT, score FLOAT8, data JSONB)
```

### Triggers

Automatically sync row changes to Antfly (useful when not using the index AM):

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

### Status Check

```sql
SELECT antfly_status('http://localhost:8080');
```

## Project Structure

```
src/
├── lib.rs            # Extension entry point + _PG_init
├── client.rs         # Antfly HTTP client
├── functions.rs      # SQL functions (antfly_search, antfly_status)
├── triggers.rs       # Trigger function (antfly_sync_trigger)
└── index_am/
    ├── mod.rs        # AM handler (IndexAmRoutine)
    ├── ctid.rs       # ctid ↔ document ID encoding
    ├── options.rs    # WITH clause parsing (url, collection)
    ├── build.rs      # ambuild, ambuildempty, aminsert
    ├── scan.rs       # ambeginscan, amrescan, amgettuple, amendscan
    ├── vacuum.rs     # ambulkdelete, amvacuumcleanup
    ├── cost.rs       # amcostestimate
    └── operator.rs   # @@@ operator and SQL registration
```

## Testing

```bash
cargo pgrx test
```

## License

See [LICENSE](LICENSE).
