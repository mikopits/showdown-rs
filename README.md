showdown-rs
===========

A very experimental Rust chatbot library for
[Pokemon Showdown](https://pokemonshowdown.com).

Usage
-----

Clone the repo, write your own plugins, run `cargo run`.
See example configuration file, and example plugins provided in the plugin mod
to get started.

NOTE: The `config.toml` and `.env` files should be placed in the root directory
of the project.

Todos
-----

*Hard*
* Static lifetimes on websocket messages seem like the wrong thing to do.
* Spawn new threads for every plugin `handle`.
* Implement plugins that operate on a timer.
* Reduce necessary LOC to create functioning plugins.
* Automatic matching for plugins with `Config` options `plugin_prefixes` and
`case_insensitive`. Approach idea: provide a raw string when a new plugin is
created to be used as a regex match. Write a default implementation of
`is_match` using the bot config and this provided string. Use `RegexBuilder`.

*Easy but annoying*
* Expand API for `Message`.
* Tests and documentation.
