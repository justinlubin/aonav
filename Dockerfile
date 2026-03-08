FROM alpine:latest

RUN apk add --no-cache uv
RUN apk add --no-cache cargo
RUN apk add --no-cache build-base
RUN apk add --no-cache clang21-libclang

RUN wget -P /usr/local/bin https://github.com/meelgroup/ganak/releases/download/release%2F2.5.3/ganak-linux-amd64.zip
RUN cd /usr/local/bin/ && unzip ganak-linux-amd64.zip && rm -rf ganak-linux-amd64.zip include/ lib/

WORKDIR /home/default

COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN cargo install --root /usr/local --path .
RUN rm -rf Cargo.toml Cargo.lock src/ target/ ~/.cargo
RUN apk del cargo

COPY benchmark benchmark


# CMD ["/bin/sh", "/home/default/benchmark/artifact-eval.sh"]

# FROM alpine:latest
# COPY --from=builder /home/aonav-builder/target/release/aonav /usr/local/bin
# 
# WORKDIR /usr/src/myapp

# RUN apt-get update
# RUN apt-get -y install build-essential
# RUN apt-get -y install curl
# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# 
# WORKDIR /home/ubuntu/
# COPY . .
# 
# RUN /root/.cargo/bin/cargo install --path .

# FROM alpine:latest

# # Install dependencies
# RUN apk add build-base
# RUN apk add --no-cache cargo
# 
# # Set up working directory
# RUN mkdir /home/aonav-user
# WORKDIR /home/aonav-user/
# 
# # Copy over files
# COPY . .
