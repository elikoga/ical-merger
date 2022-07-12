# ical-merger

ical-merger is a tool to merge multiple iCalendar files from the web into one.

It serves the files as a web-service, so you can subscribe to the calendar in your calendar application.

## Setting it up

Currently the best way to set it up is to install the application using cargo:

```bash
cargo install --git https://github.com/elikoga/ical-merger
```

From the [Rust Book](https://doc.rust-lang.org/book/ch14-04-installing-binaries.html):

> If you installed Rust using rustup.rs and don’t have any custom configurations, this directory will be `$HOME/.cargo/bin`. Ensure that directory is in your `$PATH` to be able to run programs you’ve installed with `cargo install`.

This adds the `ical-merger` binary to your `$PATH`.

On running the server like this:

```bash
ical-merger
```

The program will look for `config.yaml` and if not found `config.json` in the current working directory.

An example config file can be found in the repository [`./example.config.yaml`](./example.config.yaml).

You can see one way to more comfortably generate configuration files by looking at [`./generate_config_example.py`](./generate_config_example.py).

You can configure the web server [`rocket`](https://rocket.rs/) as documented in its [`config`](https://rocket.rs/v0.5-rc/guide/configuration/) section.
Most notably, the port can be set with `ROCKET_PORT=<PORT>` as an environment variable with a default of `8000`.

```bash
ROCKET_PORT=8000 ical-merger
```

By default, we pull in the latest versions of all calenders every minute. This is currently hard-coded in the program.
