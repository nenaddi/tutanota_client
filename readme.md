<!--
Copyright 2019 Fredrik Portström <https://portstrom.com>
This is free software distributed under the terms specified in
the file LICENSE at the top-level directory of this distribution.
-->

# Tutanota client

This is an unofficial thin wrapper over the [Tutanota](https://tutanota.com) encrypted email service, using [Hyper](https://hyper.rs). It provides a simple idiomatic Rust API for each remote procedure as well as cryptographic functions. It is in an early stage of development, currently supporting only the most basic features.

The intention of making a thin wrapper is that a thick wrapper can be made as a separate crate and added on top of it, doing things such as handling caching, retrying requests and maintaining sessions, periodical updates, databases of email and search indexes. On top of the thick wrapper, a UI can be added as yet another separate crate.

See the example program. It can be run with the command `cargo run --example example *email_address*`. It takes an email address as a command line argument and a password on the console, signs in, retreives a list of email, decrypts and displays their subject lines, and retreives the body and first attachment of the first email in the list, and decrypts and displays them. In the lists of sessions found in the login settings, the example program is displayed as “Rust”.
