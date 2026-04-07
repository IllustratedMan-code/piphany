A pipeline manager written in steel/rust

## Running the repl

After cloning the repo, you can run this in the root of the repository:

```shell
cargo run -- --repl
```

This will load [main.scm](./src/steel-modules/main.scm), an example pipeline.

Check [derivations.scm](./src/steel-modules/derivation.scm) for functions added by piphany. Anything used in the provide statement is valid.

Also, check out the [Steel repo](https://github.com/mattwparas/steel) for more instructions on how the scheme implementation works.
