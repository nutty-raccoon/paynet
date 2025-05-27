# Contributing a PR 🥇 

Hello 👋 ! Thank you for your interest in contributing to a project!

Here is a typical workflow for contributing... if you have *any* questions, please ask!

0. Decide what you want to work on (if you want to fix an issue, please comment on the issue asking us to assign it to you)
1. [Fork][fork] the repository you want to work on
2. [Clone][clone] the forked repository locally
3. [Create a branch][branch]
    - We recommend starting your branch name with the issue number you are working on
    - We also recommend giving your branch a helpful name
    - A good branch name is `7-fix-lint-errors`; a bad one is `fix-stuff`
4. Make your code changes
    - This is the really fun part 😃
5. Write tests
    - Another fun part as you get to write some tests to show off your work 🚀
6. Push your changes to your fork
7. [Create a pull request][pr] from your fork to the original repository
8. We'll review the updates you made and merge the PR!

# Local Development 🐳

This section describes how you can test, lint, and explore a project.

## Prerequisites 📝  

If you want to test, lint, or explore a project, make sure you have [docker][docker] and [docker-compose][docker-compose] installed (if you don't see: [installing docker][docker-install]).

Then you can use the **test**, **lint**, and **dev** docker compose services listed below!

## Run the node 🧮

For most local development use, you don't want to actually interact with the blockchain, meaning the mint and melt operation are instantly considered paid. This is the by-default config for the project dockerfile.

To run the node with its dependencies (postgres database and signer service) run:

```shell
$  docker compose -f docker-compose.observability.yml -f docker-compose.testnet.yml -f docker-compose.app.yml up -d
```

It will automatically set up a functional dev environment.
When working on a specific part of the infra, shut down the container you are debuging and run in localy instead.
You just have to update the url accordingly (eg. `SIGNER_URL=http://signer:10001` becomes `SIGNER_URL=http://localhost:10001`).

## Test a Project 🧪

### Unit tests 🔍

For basic unitest, run the following command from the root directory of the project:

```shell
cargo test
```

### Lint a Project 🧹

To lint a project, run the following command from the root directory of the project:

```shell
cargo clippy
```

[fork]: https://docs.github.com/en/github/getting-started-with-github/fork-a-repo
[clone]: https://docs.github.com/en/github/creating-cloning-and-archiving-repositories/cloning-a-repository
[branch]: https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging
[pr]: https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request-from-a-fork
[docker-compose]: https://docs.docker.com/compose/
[docker-install]: https://docs.docker.com/get-docker/
[docker]: https://www.docker.com/get-started
