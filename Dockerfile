FROM rust:latest

RUN apt-get update -yqq && apt-get install -yqq cmake g++
RUN mkdir /tmp/sockets

ADD ./ /app
WORKDIR /app

RUN chown 1001 /app \
    && chmod "g+rwX" /app \
    && chown 1001:root /app

RUN mkdir -p /tmp/sockets && chown -R 1001:root /tmp/sockets && chmod -R 770 /tmp/sockets

RUN mkdir -p /var/run/postgresql && chown -R 1001:root /var/run/postgresql && chmod -R 770 /var/run/postgresql

#RUN cargo clean
#RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

EXPOSE 80 8081 8082
USER 1001

COPY --chown=1001:root ./target/release/crebito-ntex ./crebito-ntex

ENTRYPOINT ./crebito-ntex
