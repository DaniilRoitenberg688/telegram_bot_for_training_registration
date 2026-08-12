# ==== Этап сборки ====
FROM rust:latest AS builder

WORKDIR /app

# Устанавливаем musl-таргет и инструменты, включая те, что нужны для сборки OpenSSL из исходников
RUN apt-get update && \
    apt-get install -y musl-tools musl-dev pkg-config perl make gcc && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

# Сначала копируем только манифесты — для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./

# Трюк для кэша: создаём фиктивный main.rs, чтобы собрать зависимости отдельно
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# Теперь копируем настоящий исходный код
COPY src src

# Пересобираем уже с реальным кодом (зависимости берутся из кэша)
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# ==== Финальный минимальный образ ====
FROM alpine:3.20

WORKDIR /app

# Нужно для HTTPS-запросов (например, к Telegram API)
RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tg_trainer ./tg_trainer

RUN chmod +x ./tg_trainer

CMD ["./tg_trainer"]
