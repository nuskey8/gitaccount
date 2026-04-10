# gitaccount
 
A CLI tool to manage multiple git accounts

[![Crates.io version](https://img.shields.io/crates/v/gitaccount.svg?style=flat-square)](https://crates.io/crates/zerompk)

## Overview

gitaccount is a CLI tool for managing the user names and email addresses configured in git config.

When managing separate git accounts for work, university, or other purposes, there is a risk of committing with the wrong name or email address. gitaccount makes it possible to switch these settings with a single command.

## Installation

Install the latest binary from [Releases](https://github.com/nuskey8/gitaccount/releases).

You can also install it via cargo.

```bash
$ cargo install gitaccount
```

## Usage

```
Usage: gitaccount <COMMAND>

Commands:
  create  Create a new account profile
  edit    Edit an existing account profile
  delete  Delete an account profile
  switch  Switch git global config
  list    List configured accounts
  logout  Clear git global config
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## .gitaccount

The created account data is saved in `~/.gitaccount` in TOML format.

```toml
[accounts.foo]
name = "foo"
git_name = "foo"
email = "foo@example.com"

[accounts.bar]
name = "bar"
git_name = "bar"
email = "bar@example.com"
```

## License

This library is under the [MIT License](LICENSE).