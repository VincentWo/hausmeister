# The Rust API of Hausmeister

For development you probably just want to use the default Redis + Postgresql using:

```bash
docker-compose up -d
cp config.toml.template config.toml
cargo run
```
This let's the server listen on `[::]:3779` and allows CORS-Request from any localhost origin.
If you want to properly deploy to production you probably want to disallow CORS-Request from localhost
and allow the origins of your frontend deployment and configure your redis + postgresql URL using the config.toml.

Have Fun!
