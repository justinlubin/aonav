FROM ubuntu:latest

WORKDIR /home/ubuntu/

# Install base system components
RUN apt-get update
RUN apt-get -y install build-essential
RUN apt-get -y install curl
RUN apt-get -y install unzip
RUN apt-get -y install vim

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Install ganak
RUN curl -L https://github.com/meelgroup/ganak/releases/download/release%2F2.5.3/ganak-linux-amd64.zip > /usr/bin/ganak.zip
RUN unzip /usr/bin/ganak.zip

# Copy over files
COPY . .

# Load bash in home directory
CMD ["bash"]
