# Modos Glider API

Common API for interacting with e-ink displays driven by the Modos display
controller, which allows for fast refresh of e-ink displays. This repository
exposes a USB HID-based interface for Rust, Python, and C using 
[maturin](https://www.maturin.rs/) and 
[cbindgen](https://github.com/mozilla/cbindgen). It also include documentation
generation using [Sphinx]() for Python and rustdoc for Rust. C documentation is
provided in the header file.

Note: this package isn't yet installable as a wheel for python (e.g. via `pip`)
or as a crate for Rust (e.g. via `cargo`) so you'll need to build it yourself.

## Python 
To install the API, simply clone this directory and run:
```shell
cd glider-api
pip install .
```

To generate the documentation, you'll need to set up a virtual environment with
`glider-api` installed, and then run one of the Sphinx `make` options:
```shell
python -m venv ./venv
maturin develop
make html
```

For Windows, replace the final command with `.\make.bat html`.

## Rust and C
To create the Rust library and its C bindings, simply run `cargo build`. 

To generate Rust documentation, simply run `cargo doc`.