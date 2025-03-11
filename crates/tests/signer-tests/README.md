# Signer service integration tests

This crates contains the integration tests for the signer service.

## How to run?

### Setup environment variables

Create a `signer.env` at the root of the file and add fields according to the [example file](./.env.example/signer.env.example).

### Launch the signer server

#### Manualy

```shell
$ cargo run --bin signer
```

or

```shell
$ cargo run --release --bin signer
```

The first one will use the environment variable defined in a `signer.env` file you must create,
while the second requires that you set those variables manualy.

#### Alone using docker

##### Build

```shell
$ docker build --tag signer --file dockerfiles/signer.Dockerfile .
```

##### Run

```shell
$ docker run \
  -p 10000:10000 \
  --rm \
  -e SOCKET_PORT="10000" \
  -e ROOT_KEY="tprv8ZgxMBicQKsPeb6rodrmEXb1zRucvxYJgTKDhqQkZtbz8eY4Pf2EgbsT2swBXnnbDPQChQeFrFqHN72yFxzKfFAVsHdPeRWq2xqyUT2c4wH" \
  --name signer-server \
  -d signer-server
```

Note that you will have to manually pass the required environment variables.

#### With the other services using docker-compose

```shell
$ docker-compose -p paynet -f ./docker-compose.yml up -d
```

This time the environment variables are set in the `docker-compose.yml` file at the repository's root.

### Run the tests

```shell
$ cargo test -p signer-tests
```

The tests will read the `SOCKET_PORT` environment variable to contact the signer at `https://localhost:$SOCKET_PORT`.
It should be defined accordingly with the port exposed by your running instance of the signer service.
