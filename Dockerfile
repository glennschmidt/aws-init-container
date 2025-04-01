FROM rust:1-alpine AS build
ARG TARGETARCH
RUN apk add musl-dev
RUN mkdir -p /build
WORKDIR /build
ADD ./Cargo.* /build/
ADD ./src /build/src
RUN cargo build --release

FROM alpine:3
ARG TARGETARCH
LABEL authors="Glenn Schmidt"
COPY --from=build /build/target/release/aws-init-container /
ENTRYPOINT ["/aws-init-container"]
