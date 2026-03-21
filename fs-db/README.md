# fs-db

SeaORM database abstraction with Entity/Repository traits and a WriteBuffer.

## Features

| Feature    | Default | Description               |
|------------|---------|---------------------------|
| `sqlite`   | yes     | SQLite backend via SQLx   |
| `postgres` | no      | PostgreSQL backend via SQLx |

## Usage

```rust
use fs_db::{Entity, Repository};
// See src/lib.rs for full API details.
```
