# cachecache: A simulator for cache accesses

This program simulates a single level of accesses to a memory cache with given memory addresses.

Example:

```sh
cachecache [filename]
```

This will open up the GUI application with the file `filename` selected, if given.

File format is subject to change.

# Building

To build this application you need to be set up for gtk4 development. For further details, check out the [GTK + Rust development book](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html).

After having made sure you have GTK4 >= 4.10 installed, use `cargo` to build the project:
```sh
cargo build [--release]
```

# Merge requests

Want to improve this program? Feel free to send a bug report or merge request.

# License

`cachecache` is free software licensed under the AGPLv3 or later.
