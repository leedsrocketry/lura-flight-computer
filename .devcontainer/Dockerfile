# Dev environment for the whole repo:
#   - stable Rust for `proto` and `ground`
#   - the Espressif Xtensa Rust toolchain ("esp") for `onboard`
#   - espflash for building flash images / flashing (where USB is available)
FROM mcr.microsoft.com/devcontainers/rust:1

# Versions match what the project was set up with; bump them together.
ARG ESPUP_VERSION=0.19.0
ARG ESPFLASH_VERSION=4.7.0
ARG ESP_TOOLCHAIN_VERSION=1.97.0.0

# libudev is needed by espflash to talk to USB serial ports.
RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends libudev-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

USER vscode

# cargo-binstall downloads prebuilt binaries instead of compiling them (much faster).
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
        | bash \
    && cargo binstall -y --locked "espup@${ESPUP_VERSION}" "espflash@${ESPFLASH_VERSION}"

# Install the Xtensa toolchain for the ESP32-S3 only.
# espup writes an export script that adds the Xtensa GCC linker to PATH. Its paths
# contain version numbers, so symlink them to fixed locations and set ENV below;
# that way every process (terminals, rust-analyzer, tasks) sees them without
# needing to source the script.
RUN espup install \
        --targets esp32s3 \
        --toolchain-version "${ESP_TOOLCHAIN_VERSION}" \
        --export-file /home/vscode/export-esp.sh \
    && . /home/vscode/export-esp.sh \
    && ln -s "$(dirname "$(command -v xtensa-esp32s3-elf-gcc)")" /home/vscode/.esp-bin \
    && if [ -n "${LIBCLANG_PATH:-}" ]; then ln -s "${LIBCLANG_PATH}" /home/vscode/.esp-libclang; fi

ENV PATH="/home/vscode/.esp-bin:${PATH}" \
    LIBCLANG_PATH="/home/vscode/.esp-libclang"
