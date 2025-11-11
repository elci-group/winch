# 🛠️ Winch — Automatic Cargo Dependency Resolver

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
git clone https://github.com/elci-group/winch.git
cd winch
cargo build --release
