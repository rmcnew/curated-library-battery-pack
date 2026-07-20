# curated-library-battery-pack

providing generally useful libraries and tools needed for Rust software development

## What is this?

"Il meglio è l'inimico del bene." \--Voltaire
"Perfect is the enemy of good."

Compared to the standard libraries of [C++](https://cppreference.com/cpp/standard_library), [Go](https://pkg.go.dev/std), [Python](https://docs.python.org/3/library/index.html), or [Java](https://docs.oracle.com/en/java/javase/25/docs/api/index.html), the [Rust Standard Library](https://doc.rust-lang.org/std/) limited.  Many common programming tasks require third-party crates because the Rust Standard Library lacks the needed functionality.  Rust developers must research many third-party crates, perform a careful analysis of candidate crates, select whatever candidate crate seems to be the best, and [hope that the crate they selected will be maintained](https://youtu.be/nOSxuaDgl3s?t=2954&si=Tn1RTB3ferbu3gCv) for the lifetime of the software they are building.  Despite all of the advantages the Rust programming language offers, its limited standard library and sprawling crate ecosystem can be an impediment to the adoption of Rust in enterprise software.  Switching to a different programming language like Rust has a high cost compared to continuing to use existing enterprise-friendly programming languages like Go or Java.  The uncertainty of the Rust crate ecosystem and lack of enterprise support can be deciding factors in not moving to Rust.

What is to be done?  It is difficult to get consensus on the best way to address this issue.  Many developers and organizations are content with the current status while others [seek a large battle-tested standard library](https://github.com/rust-lang/rfcs/pull/3810) as seen in other programming languages.  But why should the Rust Standard Library be special?  It is [not dependency free](https://github.com/CAD97/blog/discussions/2) and is built from the same crates ecosystem that all Rust software uses.  [Battery Packs](https://battery-pack-rs.github.io/battery-pack/) offer an interesting middle ground to make the Rust ecosystem more friendly.  Rust community members and enterprises can craft and share their own "extended standard libraries" to fill in functionality gaps or create domain-specific libraries of crates.

Enterprise software developers want stability, long-term support, and ready-to-use tools.  Battery Pack-powered extended standard libraries can help Rust community members and enterprises to efficiently navigate the Rust crate ecosystem and turbo-charge the Rust development experience.

The `curated-library-battery-pack` is an experiment to use a Battery Pack as an "extended standard library" for Rust software development.  Please provide feedback, bug reports, and feature requests as GitHub Issues.

## Usage

```sh
cargo bp add curated-library-battery-pack
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
