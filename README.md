<p align="center">
  <img src="assets/logo.png" alt="Rootkit Signal Hunter" width="300" />
</p>

<p align="center">
  <img src="https://github.com/bcoles/rootkit-signal-hunter/actions/workflows/tests.yml/badge.svg" alt="Build Status"/>
  <img src="https://img.shields.io/github/v/release/bcoles/rootkit-signal-hunter" alt="Release"/>
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"/>
</p>

<h1 align="center">Rootkit Signal Hunter</h1>
<p align="center"><i>Detect Linux rootkits which use signals to elevate process privileges.</i></p>

---

Some rootkits install signal handlers which listen for specific signals to elevate privileges.
This tool can identify these rootkits by sending signals and observing UID switching to root.
Optionally spawns a root shell.

<p align="center">
  <img src="assets/output.gif" alt="Rootkit Signal Hunter" />
</p>

Tested with:

* [Singularity](https://github.com/MatheuZSecurity/Singularity) 5b6c4b6 (2025-10-19) on Ubuntu 24.04 kernel 6.8.0-31-generic (x64)
* [Diamorphine](https://github.com/m0nad/Diamorphine) 2337293 (2023-09-20) on Ubuntu 22.04 kernel 5.19.0-38-generic (x64)
* [Codeine](https://github.com/diego-tella/Codeine) 9644336 (2025-09-02) on Ubuntu 22.04 kernel 5.19.0-38-generic (x64)
* [KoviD](https://github.com/carloslack/KoviD) 9b67e46 (2025-10-14) on Ubuntu 24.04 kernel 6.8.0-31-generic (x64)
  * (successful detection requires knowledge of hardcoded target PID `666`)


## Installation

You can download the latest pre-built binaries from the [Releases page](https://github.com/bcoles/rootkit-signal-hunter/releases);
or build the latest pre-release version from source:

```sh
git clone https://github.com/bcoles/rootkit-signal-hunter && \
cd rootkit-signal-hunter && \
cargo build --release
```

## Usage

```sh
rootkit-signal-hunter -- [OPTIONS]
```

## Options

- `--min <MIN>`: Minimum signal number (default: `0`)
- `--max <MAX>`: Maximum signal number (default: `64`)
- `-s`, `--shell`: Launch a root shell (if detected)
- `-t`, `--threads`: Number of worker threads (default: `16`)
- `-p`, `--pid`: Process ID to send signals to (default: `$$`)
- `-v`, `--verbose`: Enable verbose output

> [!NOTE]
> The `--pid` implementation uses `$$` to represent the process ID of a newly
> spawned process. This will fail on non-POSIX compliant shells such as Fish.

## Example

Rootkits such as [Singularity](https://github.com/MatheuZSecurity/Singularity)
and [Diamorphine](https://github.com/m0nad/Diamorphine) allow privilege
escalation using any process ID and can be trivially detected with default options
(`-s` spawns a root shell):

```sh
rootkit-signal-hunter -s
```

Rootkits such as [KoviD](https://github.com/carloslack/KoviD) require a specific
process ID to be provided with the `-p` / `--pid` flag:

```sh
rootkit-signal-hunter -s --pid 666
```

> [!NOTE]
> This risks terminating the legitimate process with ID `666`
> (if the current user has the necessary permission).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgements

Shoutout to David Reguera Garcia ([Dreg](https://github.com/therealdreg)) who implemented similar
signal-based detection as part of [lsrootkit](https://github.com/therealdreg/lsrootkit).

## Copyright

Copyright © 2025, bcoles
