![Build](https://github.com/codehag/jsparagus/workflows/Build/badge.svg)

# jsparagus - A JavaScript parser written in Rust

jsparagus is intended to replace the JavaScript parser in Firefox.

Current status:

*   jsparagus is not on crates.io yet. The AST design is not stable
    enough.  We do have a build of the JS shell that includes jsparagus
    as an option (falling back on C++ for features jsparagus doesn't
    support). See
    [mozilla-spidermonkey/rust-frontend](https://github.com/mozilla-spidermonkey/rust-frontend).

*   It can parse a lot of JS scripts, and will eventually be able to parse everything.
    See the current limitations below, or our GitHub issues.

*   Our immediate goal is to [support parsing everything in Mozilla's JS
    test suite and the features in test262 that Firefox already
    supports](https://github.com/mozilla-spidermonkey/jsparagus/milestone/1).

Join us on Discord: https://discord.gg/tUFFk9Y


## Getting started

Part of jsparagus is generated by a Python script, so start by setting
up your Python environment, make sure `cargo-fmt` is installed, then
use `make all` to build the parser:

```sh
python3 -m venv venv
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt

cargo install cargo-fmt

make all
```

(**Note:** This takes about 3 minutes to run on my laptop. The Python
script is slow. We're working on it!)

When it's done, you can:

*   Run `make check` to make sure things are working.

*   `cd rust && cargo run` to test the JS parser and bytecode emitter.


## Limitations

It's *all* limitations, but I'll try to list the ones that are relevant
to parsing JS.

*   Lookahead assertions are limited to one token. (The JS grammar
    contains an occasional
    ``[lookahead != `let [`]``
    and even
    ``[lookahead != `async [no LineTerminator here] function`]``.)

*   Restricted productions ("`[no LineTerminator here]`") are not implemented yet.

*   Error messages are poor.

*   No table compaction or table optimization. There's plenty of
    low-hanging fruit there.
