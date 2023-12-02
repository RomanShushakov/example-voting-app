FROM --platform=$BUILDPLATFORM rust:1.74 AS buildbase
RUN rustup target add wasm32-wasi
WORKDIR /src

FROM --platform=$BUILDPLATFORM buildbase AS buildclient
COPY . /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    cargo build --target wasm32-wasi --release

FROM scratch AS client
ENTRYPOINT [ "result_wasm.wasm" ]
COPY --link --from=buildclient /src/target/wasm32-wasi/release/result_wasm.wasm  result_wasm.wasm
