<img align="right" width="128" height="128" src="logo.png">

<h1>SPIS</h1>

[![CI](https://github.com/gbbirkisson/spis/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/gbbirkisson/spis/actions/workflows/ci.yml)

This project is called "Simple Private Image Server" or "SPIS" for short. It's purpose is to be a lightweight and fast server to display images hosted on a private server.

- [Development](#development)
  - [Setup test dependencies](#setup-test-dependencies)
  - [Fetching test data](#fetching-test-data)
  - [Running](#running)

## Development

### Setup test dependencies

```console
$ make setup
```

### Fetching test data

```console
$ make dl-img
```

### Running

Run stack with:

```console
$ make dev
```

Or alternatively open 3 terminals and run:

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