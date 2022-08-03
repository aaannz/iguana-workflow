# Iguana workflow

_part of the Iguana research project_

Implementation of iguana workflow parser. Iguana workflow is a YAML document loosely based on GitHub workflow YAML designed to specify order and dependencies between different containers. See [examples](examples) for example usage.

## Usage

### Building

To build the tool you will need rust 2021 edition (v1.56 and newer) and related cargo binary.

```
git clone https://github.com/aaannz/iguana-workflow
cd iguana-workflow
cargo build
```

To build release target use
```
cargo build --release
```

For quick testing use `cargo run -- --dry-run --log-level=debug workflow_file`.

See `iguana-workflow --help` for complete argument overview.

## Testing

Tool is designed to be run as part of the iguana initrd, however for testing it can be run on normal system as well. VM system is strongly recommended as iguana-workflow runs containers in privileged mode by default.

Log level can be set either by using `--log-level` option or using `RUST_LOG=debug` environmental variable.

Use `--dry-run` together with `--log-level` to see what iguana-workflow would do based on provided workflow yaml file.

## Workflow syntax

See [workflow syntax overview](workflow.md) for details about workflow file.