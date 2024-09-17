FROM --platform=$BUILDPLATFORM rust:slim AS builder-base

ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN apt-get update -qqqqy && apt-get install -qqqqy --no-install-recommends pkg-config libssl-dev librust-openssl-dev

FROM --platform=$BUILDPLATFORM builder-base AS builder

RUN BUILD_OPTIONS=""; TARGET="" \
  && if [ "${TARGETPLATFORM}" != "${BUILDPLATFORM}" ]; then \
        case "${TARGETPLATFORM}" in \
            'linux/arm64') \
                apt-get update -qqqqy \
                apt-get install -qqqqy --no-install-recommends g++-aarch64-linux-gnu libc6-dev-arm64-cross crossbuild-essential-arm64  \
                TARGET="aarch64-unknown-linux-gnu" \
                BUILD_OPTIONS="--target=${TARGET}" \
                ;; \
            'linux/amd64') \
                apt-get update -qqqqy \
                apt-get install -qqqqy --no-install-recommends g++-x86_64-linux-gnu libc6-dev-amd64-cross crossbuild-essential-amd64 \
                TARGET="x86_64-unknown-linux-gnu" \
                BUILD_OPTIONS="--target=${TARGET}" \
                ;; \
            *) \
                ;; \
        esac \
 &&   rustup target add ${TARGET} \
 &&   rustup toolchain install stable-${TARGET} \
 ;  fi \
 ; export BUILD_OPTIONS

WORKDIR /root

RUN cargo new --bin work
 
WORKDIR /root/work

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release ${BUILD_OPTIONS}
RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --release ${BUILD_OPTIONS}

FROM buildpack-deps:curl

COPY --from=builder --chown=1000:1000 /root/work/target/release/sqlx-based /usr/local/bin/sqlx-based

ENTRYPOINT [ "/usr/local/bin/sqlx-based" ]
