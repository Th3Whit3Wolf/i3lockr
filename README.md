# i3lockr

Distort a screenshot and run `i3lock`.

## Quick start [[Documentation]](USAGE.md)

Signed binary releases are availible on the [releases page](https://github.com/owenthewizard/i3lockr/releases).

Or build it yourself:
```bash
git clone --depth=1 git://github.com/owenthewizard/i3lockr.git && cd i3lockr
cargo build --release # you may adjust features here
sudo strip -s target/release/i3lockr -o /usr/local/bin/i3lockr
i3lockr --blur 25 -- --nofork --ignore-empty-password # use your favorite args
```

## Screenshots

Without `--blur`
![screenshot without blur](.github/blur-0.png)

With `--blur=10`
![screenshot with blur 10](.github/blur-10.png)

With `--blur=25`
![screenshot with blur 25](.github/blur-25.png)

`i3lockr` (since v1.0.0) is incredibly fast at all blur levels, try timing it yourself with `time`.

## Important Notes

`i3lockr` always exits with `EXIT_SUCCESS`. This means that commands such as `i3lockr && systemctl suspend` may not lock the screen if there was an error.

### Coding Style

Obey `rustfmt` and Rust 2018 conventions.

## Contributing

Pull requests are always welcome.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the terms of both the MIT License and the Apache License (Version 2.0).

## Versioning

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Changes are documented in the [Changelog](CHANGELOG.md).

See the [tags](https://github.com/owenthewizard/i3lockr/tags) for available releases.

## Authors

See [the list of contributors](https://github.com/owenthewizard/i3lockr/contributors).

## License

See [LICENSE-APACHE](LICENSE-APACHE.md) and [LICENSE-MIT](LICENSE-MIT.md) for details.

## Acknowledgments

* [i3lock](https://github.com/i3/i3lock) by [Michael Stapelberg](https://github.com/stapelberg) and [contributers](https://github.com/i3/i3lock/graphs/contributors)
* [i3lock-fancy](https://github.com/meskarune/i3lock-fancy) by [Dolores Portalatin](https://github.com/meskarune) for inspiration.
* [Martin Dørum](https://github.com/mortie) for contributions to `i3lock` that made this possible.
