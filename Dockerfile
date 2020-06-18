FROM debian:buster-slim
RUN apt-get update
COPY ./target/release/shorty-rs /usr/bin/shorty-rs/shorty-rs
COPY ./page /usr/bin/shorty-rs/page


WORKDIR /usr/bin/shorty-rs
RUN ["chmod", "+x", "./shorty-rs"]
ENV SHORTY_PORT 80
ENV SHORTY_BASE_URL https://www.matthiaskind.com/s
ENV SHORTY_DB_PATH /mnt/db/database.sqlite
EXPOSE 80
ENTRYPOINT ["./shorty-rs"]