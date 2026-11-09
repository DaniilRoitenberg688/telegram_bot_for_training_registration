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

Всё описано в корневом `docker-compose.yaml`:
- `bot` — Rust-бот и API (наружу не публикуется, доступен только внутри docker-сети);
- `web` — Caddy: сам получает HTTPS-сертификат Let's Encrypt, раздаёт собранный фронт
  и проксирует `/api` на бота. Слушает 80 и 443.

1. Направьте домен (A-запись) на IP сервера. Подойдёт и бесплатный поддомен (DuckDNS и т.п.).
2. В корневой `.env` на сервере добавьте `DOMAIN=trainer.example.com`.
3. `docker compose up -d --build`
4. BotFather: `/mybots` → бот → Bot Settings → Menu Button → `https://trainer.example.com`
   Это делается один раз, `npm run tg` на сервере не нужен.

Порты 80 и 443 на сервере должны быть свободны и открыты в фаерволе: 80 нужен Let's Encrypt
для выдачи сертификата.
