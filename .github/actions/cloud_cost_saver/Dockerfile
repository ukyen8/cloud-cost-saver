FROM rust:latest

# Create a new directory for the action
WORKDIR /action

# Copy the current directory contents into the container
COPY . .

# Install any necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build the project
RUN cargo build --release

# Copy entrypoint script and make it executable
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set the entrypoint to the script
ENTRYPOINT ["/entrypoint.sh"]
