FROM rust:latest AS builder

RUN mkdir -p /root/work

COPY . /root/work

WORKDIR /root/work

RUN cargo build --release

FROM buildpack-deps:curl

COPY --from=builder --chown=1000:1000 /root/work/target/release/mysql-based-api /usr/local/bin/mysql-based-api

ENTRYPOINT [ "/usr/local/bin/mysql-based-api" ]
