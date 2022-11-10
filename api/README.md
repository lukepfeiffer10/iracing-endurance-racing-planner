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
cargo lambda build --arm64
```

### 2. Running locally
Create a local docker network

```shell
docker network create -d bridge race-planner
```

Create a local postgres container for the database and run the migrations from the db crate

```shell
docker run --name planner-db --network=race-planner -p 5432:5432 -v planner-db-data:/var/lib/postgresql/data -e POSTGRES_USER=race_planner -e 'POSTGRES_PASSWORD=RacingPlanner!2' -d postgres

cd ../db
cargo run
```

Install the [AWS SAM CLI](https://github.com/aws/aws-sam-cli)

Run the following command to start a local docker container to mimic the AWS stack
```shell
sam local start-api --docker-network race-planner --warm-containers EAGER
```