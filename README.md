# Dependencies

If possible, you should use the rustup tool to install, instead of installing
cargo directly from the package manager.

```bash
sudo apt install rustup
```

If this is not possible (because it's not available in the repositories), you
can install cargo directly from the package manager:

```bash
sudo apt install cargo
```

Note that cargo will be very old if you install it from the package manager. If
you want to use the latest version, you should install rustup and use it to
install cargo.

Alternatively, you can install rustup from the official website:
```bash
wget https://sh.rustup.rs -O rustup.sh
# Review the contents of the script. If you're satisfied, run it.
chmod +x rustup.sh
./rustup.sh
```

# Compile and run with --help

```bash
cargo run -- --help
```

# Example Usage

```bash
cargo run -- ~/poc/*/*2024-02-22_13*/*chang*.sfs*
```

# Example output:

```console
$ cargo run -- ~/poc/*/*2024-02-22_13*/*chang*.sfs*
...
Start: 2024-02-16 03:01:05
End: 2024-02-22 11:59:59
Total operations: 851428
---
      Operation     Count|Average        
        UNLOCK:    208281|ops/second: 0.38
         WRITE:    206607|ops/second: 0.38
        LENGTH:    139797|ops/second: 0.25
       ACQUIRE:     94154|ops/second: 0.17
       RELEASE:     93896|ops/second: 0.17
        CREATE:     64589|ops/second: 0.12
      CHECKSUM:     16507|ops/second: 0.03
         TRUNC:      7836|ops/second: 0.01
       SETGOAL:      6061|ops/second: 0.01
        UNLINK:      4049|ops/second: 0.01
  SETTRASHTIME:      4040|ops/second: 0.01
        ACCESS:      3923|ops/second: 0.01
         PURGE:      1640|ops/second: 0.00
          MOVE:        26|ops/second: 0.00
       SESSION:        14|ops/second: 0.00
          ATTR:         4|ops/second: 0.00
        CLRLCK:         4|ops/second: 0.00
```
