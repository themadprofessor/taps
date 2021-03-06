# TAPS

[![license](https://img.shields.io/github/license/themadprofessor/taps?style=flat-square)](LICENSE)
[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![activity](https://img.shields.io/github/commit-activity/m/themadprofessor/taps?style=flat-square)](Activity)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square)](https://github.com/rust-secure-code/safety-dance/)

From the IETF TAPS Architecture draft:
> The goal of the Transport Services architecture is to provide a
> common, flexible, and reusable interface for transport protocols.  As
> applications adopt this interface, they will benefit from a wide set
> of transport features that can evolve over time, and ensure that the
> system providing the interface can optimize its behavior based on the
> application requirements and network conditions, without requiring
> changes to the applications.  This flexibility enables faster
> deployment of new features and protocols.  It can also support
> applications by offering racing and fallback mechanisms, which
> otherwise need to be implemented in each application separately.

## Table of Contents

- [Security](#security)
- [Background](#background)
- [Install](#install)
- [API](#api)
- [Contributing](#contributing)
- [License](#license)

## Security

This library is in the very early stages of development, as such has not be independently audited.
Attempts are made to ensure minimal security vulnerabilities through the use of [cargo-audit](https://github.com/RustSec/cargo-audit)
when pushes are made to the library.

**NOT READY FOR PRODUCTION**

## Background

This crate is a Rust implementation of the IETF's [TAPS](https://datatracker.ietf.org/wg/taps/documents/) API.
Key aims of the TAPS API include:
- Defining a high-level, asynchronous networking API
- Decoupling applications from transport layer implementations
- Providing a unified interface for portable network programming

## Install

Add the following to your Cargo.toml
```toml
[dependencies]
taps = { git = "https://github.com/themadprofessor/taps.git" }
```

Documentation can be built with the repo cloned and running:
```shell script
cargo doc --open
```

## API

The API relies heavily on [trait objects](https://doc.rust-lang.org/nightly/reference/types/trait-object.html?highlight=dyn#trait-objects) 
This is to allow the underlying implementation of the TAPS API to change without applications having to rewrite any code.

## Testing

Run the following to test the library:
```shell script
cargo test
```
This doesn't run the tests which special setup from the tester, for example listen_cargo requires the user to send a
request to the test (`curl -vvv --data-binary @Cargo.toml -H "Content-Type: application/toml" 127.0.0.1:8081`)

## Contributing

See [the contributing file](CONTRIBUTING.md)!

PRs accepted.

Small note: If editing the Readme, please conform to the [standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

[MIT © Stuart Reilly.](LICENSE)
