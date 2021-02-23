# Contributing to Zellij

Thanks for considering to contribute to Zellij!

**First**: if you're unsure or afraid of anything, just ask on our
[Discord server][discord-invite-link] or submit the issue or pull request anyway.
You won't be yelled at for giving it your best effort. The worst that can happen
is that you'll be politely asked to change something. We appreciate any sort
of contributions, and don't want a wall of rules to get in the way of that.

# Code of Conduct

Before contributing please read our [Code of Conduct](CODE_OF_CONDUCT.md) which
all contributors are expected to adhere to.

## Building
To work around a [Cargo bug][https://github.com/rust-lang/cargo/issues/7004], you'll need to use the included `build-all.sh` script.


```sh
# An unoptimized debug build
./build-all.sh
# A fully optimized release build
./build-all.sh --release
```

The build script has an optional dependency on `binaryen --version` > 97, for it's command `wasm-opt`.

## Looking for something to work on?

If you are new contributor to `Zellij` going through [beginners][good-first-issue]
should be a good start or you can join our public
[Discord server][discord-invite-link], we would be happy to help
finding something interesting to work on and guide through.

[discord-invite-link]: https://discord.gg/feHDHahHCz
[good-first-issue]: https://github.com/zellij-org/zellij/labels/good%20first%20issue

## Filing Issues

Bugs and enhancement suggestions are tracked as GitHub issues.

### Lacking API for plugin in Zellij?

If you have a plugin idea, but Zellij still doesn't have API required to make
the plugin consider opening [an issue][plugin-issue] and describing your requirements.

[plugin-issue]: https://github.com/zellij-org/zellij/issues/new?assignees=&labels=plugin%20system

### How Do I Submit A (Good) Bug Report?

After you've determined which repository your bug is related to and that the
issue is still present in the latest version of the master branch, create an
issue on that repository and provide the following information:

- Use a **clear and descriptive title** for the issue to identify the problem.
- Explain which **behavior you expected** to see instead and why.
- Describe the exact **steps to reproduce the problem** in as many details as necessary.
- When providing code samples, please use [code blocks][code-blocks].

### How Do I Submit A (Good) Enhancement Suggestion?

Instructions are similar to those for bug reports. Please provide the following
information:

- Use a **clear and descriptive title** for the issue to identify the suggestion.
- Provide a **description of the suggested enhancement** in as many details as necessary.
- When providing code samples, please use [code blocks][code-blocks].

[code-blocks]: https://help.github.com/articles/creating-and-highlighting-code-blocks/

## Submitting Pull Requests

Instructions are similar to those for bug reports. Please provide the following information:

- If this is not a trivial fix, consider **creating an issue to discuss first** and **later link to it from the PR**.
- Use a **clear and descriptive title** for the pull request.
    - Follow [Conventional Commit specification](https://www.conventionalcommits.org/en/v1.0.0/)
where sufficiently large or impactful change is made.
- Provide a **description of the changes** in as many details as necessary.

Before submitting your pull request, also make sure that the following conditions are met:

- Your new code **adheres to the code style** through running `cargo fmt`.
- Your new code **passes all existing and new tests** through running `cargo test`.
