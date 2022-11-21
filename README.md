<img align="right" width="128" height="128" src="logo.png">

<h1>SPIS</h1>

[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/gbbirkisson/spis)](https://github.com/gbbirkisson/spis/releases)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/gbbirkisson/spis/main)](https://github.com/gbbirkisson/spis/commits/main)
[![CI](https://github.com/gbbirkisson/spis/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/gbbirkisson/spis/actions/workflows/ci.yml)
[![codecov](https://codecov.io/github/gbbirkisson/spis/branch/main/graph/badge.svg?token=5VQHEBQ7JV)](https://codecov.io/github/gbbirkisson/spis)
[![GitHub](https://img.shields.io/github/license/gbbirkisson/spis)](https://github.com/gbbirkisson/spis/blob/main/LICENSE)

This project is called "Simple Private Image Server" or "SPIS" for short. It's purpose is to be a lightweight and fast server to display media hosted on a private server. It came about when I was searching for a solution like this and found nothing. Everything seemed way to feature heavy and slow, requiring you to setup databases and other components.

The goals for this project are:
* Simple to setup 🏝️
* Lightweight, multi-threaded and fast 🚀
* Minimalistic GUI 🤩
* Easy to use on mobile 📱

<h2>Table of contents</h2>

- [Screenshot](#screenshot)
- [Setup](#setup)
  - [Docker](#docker)
  - [Binary](#binary)
- [Progressive Web App](#progressive-web-app)
- [Development](#development)
  - [Setup dependencies](#setup-dependencies)
  - [Running](#running)

## Screenshot

This is how the GUI looks!

<img width="100%" src="screenshot.jpg">

## Setup

### Docker

Easiest way to run the server is with the docker image:

```console
$ docker run -it \
    -p 8080:8080 \
    -v /path/to/your/media:/var/lib/spis/media \
    -v /path/to/save/data:/var/lib/spis/data \
    spis # <- TODO
```

### Binary

```console
$ # TODO
```

## Progressive Web App

If you have an Android phone, you can add SPIS as a PWA. Open up the server home page in the chrome browser, open top-right menu, and select `Add to Home screen`.

## Development

### Setup dependencies

```console
$ make setup dl-img
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

And then open [http://localhost:7000](http://localhost:7000)
