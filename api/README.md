# Getting Started

### 1. Building
Run this script to get the correct compilation target:

```shell
rustup target add aarch64-unknown-linux-gnu
```

Once this is done, install `cargo-lambda`:

```shell
cargo install cargo-lambda
```


This Cargo subcommand will give you the option to install [Zig](https://ziglang.org/) to use as the linker. You can also install [Zig](https://ziglang.org/) using the instructions in their [installation guide](https://ziglang.org/learn/getting-started/#installing-zig).

Now you can run the following command to build the lambda functions

```shell
cargo lambda build --release --target aarch64-unknown-linux-gnu
```

### 2. Running locally
Install the [AWS SAM CLI](https://github.com/aws/aws-sam-cli)

Run the following command to start a local docker container to mimic the AWS stack
```shell
sam local start-api
```