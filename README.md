## Development

### Dependencies

```console
# Install nginx
$ sudo apt install nginx

# Install watchexec
$ cargo install watchexec-cli

# Install trunk
$ cargo install trunk

# Add WASM build target
$ rustup target add wasm32-unknown-unknown
```

### Fetching test data

```console
$ make dl-img
```

### Running

Open 3 terminals and run:

```console
$ make dev-nginx
```

```console
$ make dev-server
```

```console
$ make dev-gui
```

And then open [http://localhost:9000](http://localhost:9000)