FROM rust as build

WORKDIR /usr/src/git-web-view
COPY . .

RUN cargo build --release
RUN ln -s $(cd ./target/release/; pwd)/git-web-view /git-web-view

FROM gcr.io/distroless/cc

COPY --from=build /git-web-view /usr/local/bin/git-web-view

ENTRYPOINT ["git-web-view"]
