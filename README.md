# uptime-thing

A toy uptime monitoring app made as a challenge. Run with:

```sh
export RUST_LOG=info CONFIG_PATH=/path/to/config.yml
cargo run --release
```

Take a look at the [config.yml example](config.example.yml) for more info.

TODOs:

- [x] Switch to YAML for configuration.
- [ ] Add per-check timeouts.
- [ ] Store the time series in a SQLite database.
- [ ] Display real-time statistics in a web UI.
- [ ] Actually implement ping and TCP checks.
