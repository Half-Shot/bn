# bn

Simple Rust application to check the battery level and send a notification when it drops too low.

This works best as a Systemd user service which runs on a timer. An example is provided in the `contrib` directory.

```
Simple application to notify when the battery drops too low

Usage: bn [OPTIONS] --critical-percentage <CRITICAL_PERCENTAGE>

Options:
  -w, --warn-percentage <WARN_PERCENTAGE>
          When the battery drops below this level, send a warning notification
  -c, --critical-percentage <CRITICAL_PERCENTAGE>
          When the battery drops below this level, send an urgent critical notification
  -s, --serial <SERIAL>
          The serial number of the battery to check. If not provided, this command will list all batteries
  -h, --help
          Print help
  -V, --version
          Print version
```

## Building

```sh
cargo build -r
./target/release/bn -s 
```
