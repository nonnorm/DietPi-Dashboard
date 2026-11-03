# DietPi-Dashboard
A web dashboard for DietPi

## Installation
To install, use one of the [precompiled releases](#release), [nightly builds](#nightly) or [compile it yourself](#compiling)

### Downloading
#### Release:

```sh
curl -fL "$(curl -sSf 'https://api.github.com/repos/ravenclaw900/DietPi-Dashboard/releases/latest' | mawk -F\" "/\"browser_download_url\": \".*dietpi-dashboard-$G_HW_ARCH_NAME\"/{print \$4}")" -o dietpi-dashboard # Download latest binary for current architecture
chmod +x dietpi-dashboard # Make binary executable
./dietpi-dashboard # Run binary
```

#### Nightly:

```sh
curl -fL "https://nightly.link/ravenclaw900/DietPi-Dashboard/workflows/push-build/main/dietpi-dashboard-$G_HW_ARCH_NAME.zip" -o dietpi-dashboard.zip # Download latest nightly build for current architecture
unzip dietpi-dashboard.zip # Unzip binary
rm dietpi-dashboard.zip # Remove archive
chmod +x dietpi-dashboard # Make binary executable
./dietpi-dashboard # Run binary
```


### Compiling
#### Prereq:

```sh
apt install gcc
dietpi-software install nodejs git # Install Node.js (webpage) and Git (git clone)
corepack enable # Enable pnpm package manager, for node dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Install Rust (backend)
source ~/.cargo/env # Update $PATH
cargo install just # Install just command runner, for build file
```

#### Compiling:

```sh
git clone https://github.com/ravenclaw900/DietPi-Dashboard # Download source code
cd DietPi-Dashboard # Change directories
cargo build --release # Compile binary for your platform
./target/release/dietpi-dashboard # Run binary
```

Note that there will be a difference between self-compiled binaries and the nightly/release builds. The nightly/release builds are statically linked with the musl libc implementation, while self-compiled binaries will be dynamically linked with glibc. This should not affect functionality in any way.

### Open dashboard:
`http://<your.IP>:5252`

