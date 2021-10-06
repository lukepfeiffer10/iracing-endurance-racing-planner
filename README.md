# Running the app

If you haven't already, install Trunk:

```shell
cargo install trunk wasm-bindgen-cli
```

If you haven't already installed it, you need to add the wasm32-unknown-unknown target. To install this with Rustup:

```shell
rustup target add wasm32-unknown-unknown
```

Now all you have to do is run the following:

```shell
trunk serve
```


This will start a development server which continually updates the app every time you change something.