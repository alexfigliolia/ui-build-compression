## UI Build Compressor
Static file compression for UI builds powered by rust.

Creates gzip, brotli, zstd, and deflate compressed file variants along side original files for you to deploy to production.

Each algorithm is configured for the most aggressive compression settings available

### API
#### Programmatic
```rust
use ui_build_compression::compress;

compress("/path/to/my/directory");
```

#### Command Line
```bash
ui-build-compression /path/to/my/directory
```