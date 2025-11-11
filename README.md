# 🛠️ Winch — Automatic Cargo Dependency Resolver
<img width="1024" height="1024" alt="logo" src="https://github.com/user-attachments/assets/464509b0-fe8e-4ca9-bae5-9750971ca5ea" />

[![Rust](https://img.shields.io/badge/rust-1.70+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Winch is a Rust-based tool that **automatically resolves Cargo dependency conflicts** and **handles missing crates** by fetching the latest releases. It tries multiple versions and rolls back if a build fails, allowing Rust projects to recover from broken dependency states with minimal manual intervention.

---

## Features

- ✅ Detects conflicting crate versions and attempts automatic resolution.  
- 🆕 Detects missing crates, fetches latest versions, and updates `Cargo.toml`.  
- 🔄 Performs rollback if a build fails, trying multiple candidate versions (`MAX_ROLLBACKS`).  
- 🛡️ Safe testing via temporary `Cargo.winch.toml` to avoid corrupting main manifest.  
- 📊 Prints detailed logs of all attempted combinations and results.

---

## Installation

Make sure you have Rust installed (rustc 1.70+ recommended).  

Clone the repository and build:

```bash
git clone https://github.com/<your-username>/winch.git
cd winch
cargo build --release
````

The compiled binary will be at:

```
target/release/winch
```

---

## Usage

Basic usage:

```bash
./winch [--dir <path>] [--help|-h]
```

### Options

| Option         | Description                                 |
| -------------- | ------------------------------------------- |
| `--dir <path>` | Path to Rust project (default: current dir) |
| `--help`, `-h` | Show this help message                      |

---

### Example: Fixing a Broken Project

```bash
# Navigate to a project with dependency issues
cd ~/projects/my-rust-app

# Run Winch
winch --dir .
```

* Winch will attempt to build the project.
* If conflicts or missing crates are detected, it fetches candidate versions and tries builds iteratively.
* On success, `Cargo.toml` is updated with working versions.
* If all combinations fail, manual intervention is required.

---

### Example: Missing Crates Auto-Resolution

If a crate is missing from your project:

```text
error[E0463]: can't find crate for `serde_json`
```

Winch will:

1. Detect the missing crate (`serde_json`).
2. Fetch the latest release from crates.io.
3. Update `Cargo.toml` (temporarily) and attempt a build.
4. Roll back to previous versions if the build fails.

---

## Configuration

* `MAX_ROLLBACKS` (default `5`) controls how many past versions are attempted for each crate during rollback.
* Temporary testing is done in `Cargo.winch.toml` to prevent corrupting the main manifest until a successful build occurs.

---

## Limitations

* Works best with semantic versioning (SemVer) compliant crates.
* Large projects with many conflicts may result in combinatorial explosion of build attempts.
* Does not automatically resolve deeper dependency tree conflicts; manual review may be necessary in complex scenarios.

---

## Development

Clone and build in debug mode for development:

```bash
cargo build
cargo run -- --dir path/to/project
```

Add new features or improve version selection logic as needed.

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Contributions

Contributions, bug reports, and feature requests are welcome!
Please open an issue or submit a pull request on GitHub.

---

## Acknowledgements

* [Crates.io API](https://crates.io/) for fetching version data.
* `toml_edit` for safe programmatic `Cargo.toml` editing.
* `reqwest` for HTTP requests.
* `regex` for parsing Cargo build errors.
* `semver` for version comparison.
