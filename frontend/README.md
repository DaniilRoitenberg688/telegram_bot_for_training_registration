# Telegram Mini App для tg_trainer

React 19 + TypeScript + Vite. Работает поверх REST API бота (`/api/*`). Внешних UI-библиотек нет:
цвета берутся из темы Telegram (`--tg-theme-*`), «Назад» — нативная кнопка Telegram.

## Экраны

| Кто     | Экран             | API                                                             |
|---------|-------------------|-----------------------------------------------------------------|
| все     | вход              | `GET /user/{id}` (404 → «зарегистрируйтесь через /start»)       |
| клиент  | Записаться        | `GET /days?from&to`, `GET /trainings?date`, `POST /registrations` |
| клиент  | Мои записи        | `GET /registrations/{id}`, `DELETE /registrations`              |
| тренер  | Записи клиентов   | `GET /trainer/days?from&to`                                     |
| тренер  | День              | `GET /trainer/trainings?date`                                   |

Все запросы идут с заголовком `Authorization: tma <initData>`. Бэкенд (`src/api/auth.rs`) проверяет
подпись токеном бота и разрешает пользователю работать только со своими данными,
а `/trainer/*` и `PUT /training/{id}` — только тренеру.

## Структура

```
src/
  api/client.ts      — все запросы к API + ApiError
  api/types.ts       — типы, повторяющие src/models.rs
  lib/dates.ts       — даты по Москве, недели, форматирование
  lib/useAsync.ts    — загрузка данных (loading / error / success + reload)
  navigation.tsx     — стековый роутер + нативная кнопка «Назад»
  telegram.ts        — обёртка над window.Telegram.WebApp
  components/ui.tsx  — Screen, Async, Chip, Spinner...
  screens/           — экраны
```

## Разработка

Бэкенд запускается как обычно (`cargo run` в корне). Vite сам берёт `PORT` из корневого `.env`
и проксирует `/api` на `http://127.0.0.1:<PORT>`.

```bash
cd frontend
cp .env.example .env     # VITE_DEV_USER_ID = свой telegram id (для браузера)
npm install
npm run dev              # в браузере: http://localhost:5173
```

В обычном браузере нет подписанного initData, поэтому для такой разработки запускайте бэкенд
с отключённой проверкой: `API_AUTH_DISABLED=true cargo run`. Тогда фронт шлёт `X-Dev-User-Id`
из `VITE_DEV_USER_ID`. Внутри Telegram (`npm run tg`) это не нужно.

### Открыть в Telegram

```bash
npm run tg               # кнопка «Открыть» в меню бота для всех пользователей
npm run tg -- --chat <id> # кнопка только в вашем чате с ботом
```

Скрипт `scripts/telegram-dev.mjs` запускает Vite, открывает публичный HTTPS-туннель
(cloudflared, если установлен — `sudo pacman -S cloudflared`; иначе localhost.run через ssh)
и через Bot API (`setChatMenuButton`) ставит боту кнопку меню на адрес туннеля.
По Ctrl+C кнопка возвращается по умолчанию.

⚠️ Пока туннель работает, приложение и `/api` доступны из интернета по случайному адресу,
а у API нет авторизации. Для разработки это терпимо, но долго так не держите.

## Деплой на сервер

Корневой `docker-compose.yaml` поднимает два контейнера:
- `bot` — Rust-бот и API (наружу не публикуется, доступен только внутри docker-сети проекта);
- `web` — Caddy: раздаёт собранный фронт и проксирует `/api` на `bot`.
  Слушает только `127.0.0.1:${WEB_PORT}`, HTTPS делает nginx на сервере.

На одном сервере можно держать несколько ботов (например, тестовый и официальный) — каждый в своей
папке со своим `.env`:

```
                     ┌─ test.95-163-12-34.sslip.io ─▶ 127.0.0.1:8082 → web → bot (тест)
Интернет ─443─▶ nginx┤
                     └─ 95-163-12-34.sslip.io ──────▶ 127.0.0.1:8081 → web → bot (прод)
```

### 1. `.env` каждого бота

```bash
# официальный
COMPOSE_PROJECT_NAME=tg_trainer_prod
WEB_PORT=8081

# тестовый
COMPOSE_PROJECT_NAME=tg_trainer_test
WEB_PORT=8082
```

`COMPOSE_PROJECT_NAME` обязателен и должен отличаться: иначе папки с одинаковым названием Docker
считает одним проектом, и запуск одного бота пересоздаст контейнеры другого.
Остальное (`TOKEN`, `DATABASE_URL`, `TRAINER_IDS`...) у каждого бота своё.

### 2. nginx

```nginx
server {
    listen 80;
    server_name 95-163-12-34.sslip.io test.95-163-12-34.sslip.io;
    location /.well-known/acme-challenge/ { root /var/www/certbot; }
    location / { return 301 https://$host$request_uri; }
}

server {
    listen 443 ssl;
    server_name 95-163-12-34.sslip.io;                 # официальный
    ssl_certificate     /etc/letsencrypt/live/95-163-12-34.sslip.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/95-163-12-34.sslip.io/privkey.pem;
    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 443 ssl;
    server_name test.95-163-12-34.sslip.io;            # тестовый
    ssl_certificate     /etc/letsencrypt/live/95-163-12-34.sslip.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/95-163-12-34.sslip.io/privkey.pem;
    location / {
        proxy_pass http://127.0.0.1:8082;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Сертификат (сначала включите только блок на 80 порт, потом добавьте блоки 443):

```bash
sudo mkdir -p /var/www/certbot
sudo certbot certonly --webroot -w /var/www/certbot \
  -d 95-163-12-34.sslip.io -d test.95-163-12-34.sslip.io
sudo nginx -t && sudo systemctl reload nginx
```

### 3. Запуск

```bash
cd ~/tg_trainer_prod && git pull && docker compose up -d --build
cd ~/tg_trainer_test && git pull && docker compose up -d --build
curl https://95-163-12-34.sslip.io/api/ping        # → Pong
curl https://test.95-163-12-34.sslip.io/api/ping   # → Pong
```

В BotFather у каждого бота своя Menu Button: у официального `https://95-163-12-34.sslip.io`,
у тестового `https://test.95-163-12-34.sslip.io`.
