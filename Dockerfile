FROM rustlang/rust:nightly as build

RUN apt-get update -y && \
    apt-get install -y libclang-dev llvm-dev

# create a new empty shell project
RUN mkdir watcher

WORKDIR /watcher

ARG MY_GIT_TOKEN

RUN git config --global url."https://api:$MY_GIT_TOKEN@github.com/".insteadOf "https://github.com/"

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./.cargo ./.cargo

RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "\n" > src/lib.rs

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs


# copy your source tree
COPY ./ ./

# ARG DATABASE_URL
# build for release
RUN cargo build --release

# our final base
FROM gcr.io/distroless/cc

# copy the build artifact from the build stage
COPY --from=build /watcher/target/release/watcher .


# set the startup command to run your binary
CMD ["./watcher"]
