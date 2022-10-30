FROM rust:1.63.0-alpine3.16 AS build

RUN apk add musl-dev protoc

COPY . /app
WORKDIR /app

RUN cargo build --release

FROM alpine:3.16.1

COPY --from=build /app/target/release/send-message-to-kelbi /app/send-message-to-kelbi
WORKDIR /app

RUN chmod +x send-message-to-kelbi

CMD [ "./send-message-to-kelbi" ]