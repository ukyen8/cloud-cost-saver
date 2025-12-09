FROM rust:latest

# Create a new directory for the action
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Install any necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build the project
RUN cargo build --release

# Set the entrypoint to the built binary
ENTRYPOINT ["/usr/src/app/target/release/ccs"]
