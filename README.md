## UI Build Compressor

Static file compression for UI builds powered by rust.

Creates gzip, brotli, zstd, and deflate compressed file variants along side original files for you to deploy to production.

Each algorithm is configured for the most aggressive compression settings available

### API

#### Programmatic

##### Rust

```rust
// cargo add ui-build-compression

use ui_build_compression::compress;

compress("/path/to/my/directory");
```

##### Javascript/Typescript

```typescript
// yarn add ui-build-compression

import { compress } from "ui-build-compression";

void compress("/path/to/my/directory");
```

#### Command Line

```bash
ui-build-compression /path/to/my/directory
```
