FROM rust:1

WORKDIR /usr/src/app
COPY . .

# Install production dependencies and build a release artifact.
RUN cargo install --path ./cark-server

# Run the web service on container startup.
CMD ["cark-server"]
