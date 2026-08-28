# Server command

This tool was created to be used with the structure used by [mage-db-sync](https://github.com/jellesiderius/mage-db-sync)
as seen [here](https://github.com/jellesiderius/mage-db-sync/wiki/Configuring-settings-and-databases#configuring-databases).
It currently only uses the `username`, `server`, and `port` values.

## Install

```sh
brew install epenthesis/tap/server-mage-db
```

Or, if you would rather tap once and use short names afterwards:

```sh
brew tap epenthesis/tap
brew install server-mage-db
```

The installed command is `server`.

From a checkout:

```sh
cargo build --release
```

### Updating

```sh
brew update && brew upgrade server-mage-db
```

## Usage

```sh
server
server --json
server --version
```
