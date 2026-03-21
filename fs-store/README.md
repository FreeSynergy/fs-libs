# fs-store

Universal store client for FreeSynergy module registries. Fetches catalogs and
i18n bundles from HTTP or local filesystem sources with retry, TTL caching, and
offline fallback.

## Design

```
StoreSource ─── Strategy ──▶ Local(path) | Http(url)
StoreClient ─── wraps ─────▶ StoreSource + CatalogCache + DiskCache
Catalog<M>  ─── generic ───▶ any project-supplied Manifest type
```

## Quick start

```rust
use fs_store::{StoreClient, StoreSource};

// Connect to the FreeSynergy Node.Store
let mut client = StoreClient::node_store();

// Or use a local directory (for development / offline)
let mut client = StoreClient::new(StoreSource::local("./store"));

// Fetch the Node catalog (generic over your Manifest type)
let catalog: Catalog<ModuleManifest> = client.fetch_catalog("Node", false).await?;

// Find a module by ID
if let Some(pkg) = catalog.find("proxy/zentinel") {
    println!("{} v{}", pkg.name(), pkg.version());
}
```

## Implementing `Manifest`

```rust
use fs_store::manifest::{Manifest, PackageMeta};
use serde::Deserialize;

#[derive(Deserialize)]
struct ModuleManifest {
    #[serde(flatten)]
    meta: PackageMeta,
    // … project-specific fields …
}

impl Manifest for ModuleManifest {
    fn id(&self)       -> &str { &self.meta.id }
    fn version(&self)  -> &str { &self.meta.version }
    fn category(&self) -> &str { &self.meta.category }
    fn name(&self)     -> &str { &self.meta.name }
}
```

## Retry + Offline Fallback

HTTP fetches retry up to 3 times with exponential backoff (1s → 2s → 4s).
On final failure the last successful response is served from `~/.cache/fsn/store/`.
