# `dices-server` 0.3.1

This is a server to run multiple instances of `dices`, and collaborate on them.

## Running

This server needs a `PostgreSQL` instance. Set the enviroment variable `DB__URL` to the connection string
to run it.

Under the `local` directory there is a simple configuration to run a local instance. Pull up the 
`docker compose` project, and then run:
```sh
cd ./local
docker compose up -d
cargo run
```

## Configuration

The server will pull configuration from three main sources:
- The `DicesServer.toml` file, from either the current directory or an ancestor (can be changed from CLI arguments).
- The environment variables (can be deactivated from the CLI arguments).
- The CLI arguments.

For example one can insert into `DicesServer.toml`
```toml
[foo]
bar = "baz"
```
or set the `FOO__BAR=baz` environment variable (notice the double underscore), or add the `-C foo.bar=baz` argument.

To generate a valid config file one can request the default from the binary, using `cargo run -- --example-config DicesServer.toml`.
This will capture the configuration provided, so it can be used to debug the configuration used.

## API

The API is documented using [OpenApi](https://www.openapis.org/), reachable at the `/api-docs/openapi.json` endpoint. The server also serve [swagger-ui](https://swagger.io/tools/swagger-ui/)
at the `/swagger-ui` endpoint (with the default setup http://localhost:8080/swagger-ui).