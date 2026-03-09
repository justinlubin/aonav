FROM ubuntu:latest

###############################################################################
# Install dependencies

# Base
RUN apt-get update
RUN apt-get -y install build-essential
RUN apt-get -y install curl
RUN apt-get -y install unzip
RUN apt-get -y install clang
RUN apt-get -y install fonts-linuxlibertine

# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

# uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Ganak
RUN curl -L https://github.com/meelgroup/ganak/releases/download/release%2F2.5.3/ganak-linux-amd64.zip > /usr/local/bin/ganak.zip
RUN cd /usr/local/bin/ && unzip ganak.zip && rm -rf ganak-linux-amd64.zip include/ lib/

###############################################################################
# Set up PATH

ENV PATH="/root/.local/bin:/root/.cargo/bin:${PATH}"

###############################################################################
# Install aonav

WORKDIR /root

# Create aonav binary
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN cargo install --root /usr/local --path .
RUN rm -rf Cargo.toml Cargo.lock src/ target/ ~/.cargo ~/.rustup

# Set up benchmarking harness
COPY scripts/artifact-eval.sh .
COPY benchmark/entries entries
COPY benchmark/analysis analysis
RUN mkdir results

# Pre-fetch Python dependencies
RUN cd analysis && uv sync

CMD ["/bin/sh", "/root/artifact-eval.sh"]
