<!--
Copyright 2019 Fredrik Portström <https://portstrom.com>
This is free software distributed under the terms specified in
the file LICENSE at the top-level directory of this distribution.
-->

# Tutanota client

This is an unofficial thin wrapper over the [Tutanota](https://tutanota.com) encrypted email service, using [Hyper](https://hyper.rs). It provides a simple idiomatic Rust API for each remote procedure as well as cryptographic functions. It is in an early stage of development, currently supporting only the most basic features.

The intention of making a thin wrapper is that a thick wrapper can be made as a separate crate and added on top of it, doing things such as handling caching, retrying requests and maintaining sessions, periodical updates, databases of email and search indexes. On top of the thick wrapper, a UI can be added as yet another separate crate.

See the example program. It can be run with the command `cargo run --example example email_address operation`. It takes an email address as a command line argument and a password on the console. It takes an operation as the second command line argument:

- **create_folder**: Creates a mail folder with the name `Test created!`.
- **manage_folders**: Shows the list of folders with their names. Deletes the first folder with a name starting with `Test delete!`. Renames the first folder with a name starting with `Test rename!`.
- **toggle_unread**: Toggles the unread status of the last mail in the inbox.
- **view_mail**: Displays a list of mails in the inbox with their subject lines. Displays the body and the first attachment of the first mail in the inbox.

In the lists of sessions found in the login settings, the example program is displayed as “Rust”.
