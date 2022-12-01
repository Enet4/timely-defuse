# Timely Defuse

[![Timely Defuse on itch.io](https://user-images.githubusercontent.com/4738426/205147102-2ec69591-10bc-4f9d-9d4c-3211bcb09582.png)](https://e-net4.itch.io/timely-defuse)


Timely Defuse is a Web-based arcadey game written in Rust.

> On an eventful Sunday morning,
> you discover that a group of thugs are trying to
> demolish a build site illegally.
> Guess it's up to you to stop their plans for as long as you can!

## How to play

- On a mobile device,
  or a computer with a touch screen,
  touch a position to make the protagonist move to that position.
- Pick up dynamites before they explode.
- Move to bombs and disarm them before they explode.
- Don't get hit by explosions!
- Pick up the coffee to enhance the protagonist's speed and reaction times.

## Building

To run the game as a desktop application:

```sh
cargo run --release
```

To build for the web:

```sh
./build-wasm-release.sh
```

Then serve the [wasm](wasm) directory.

### Licensing and Attribution

All source code is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

Third party content and respective attribution is listed in [SOURCES.md](SOURCES.md).
All original non-code assets other than those described above
are licensed under a [Creative Commons Attribution-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-sa/4.0/).
![CC BY SA 4.0](https://i.creativecommons.org/l/by-sa/4.0/80x15.png)
