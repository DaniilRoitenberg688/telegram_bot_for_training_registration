#!/usr/bin/env node
// Запуск Mini App для открытия из Telegram в режиме разработки:
//   1. поднимает Vite (порт 5173, /api проксируется на `cargo run`);
//   2. открывает публичный HTTPS-туннель (cloudflared, если установлен, иначе localhost.run через ssh);
//   3. ставит боту кнопку меню «Открыть», ведущую на адрес туннеля;
//   4. по Ctrl+C возвращает кнопку меню по умолчанию и всё гасит.
//
// Использование:
//   npm run tg                    — кнопка меню для всех пользователей бота
//   npm run tg -- --chat 12345    — кнопка только в чате с указанным пользователем

import { spawn, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const FRONTEND_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ROOT_ENV = resolve(FRONTEND_DIR, "..", ".env");
const VITE_PORT = 5173;
const BUTTON_TEXT = "Открыть";

const chatIdx = process.argv.indexOf("--chat");
const chatId = chatIdx !== -1 ? process.argv[chatIdx + 1] : undefined;

function readEnv(path) {
  const env = {};
  for (const line of readFileSync(path, "utf8").split("\n")) {
    const m = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*?)\s*$/);
    if (m) env[m[1]] = m[2].replace(/^(['"])(.*)\1$/, "$2");
  }
  return env;
}

const { TOKEN, PORT } = readEnv(ROOT_ENV);
if (!TOKEN) {
  console.error(`TOKEN не найден в ${ROOT_ENV}`);
  process.exit(1);
}

async function telegram(method, params) {
  const res = await fetch(`https://api.telegram.org/bot${TOKEN}/${method}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(params),
    // Не даём скрипту молча зависнуть, если api.telegram.org недоступен.
    signal: AbortSignal.timeout(15_000),
  });
  const body = await res.json();
  if (!body.ok) throw new Error(`${method}: ${body.description}`);
  return body.result;
}

const setMenuButton = (menu_button) =>
  telegram("setChatMenuButton", chatId ? { chat_id: chatId, menu_button } : { menu_button });

const children = [];
function run(cmd, args, opts = {}) {
  const child = spawn(cmd, args, { stdio: ["ignore", "pipe", "pipe"], ...opts });
  children.push(child);
  return child;
}

const hasCommand = (cmd) => spawnSync("which", [cmd], { stdio: "ignore" }).status === 0;

/** Запускает туннель и резолвит его публичный https-адрес. */
function startTunnel() {
  const target = `http://localhost:${VITE_PORT}`;
  let child, pattern, name;
  if (hasCommand("cloudflared")) {
    name = "cloudflared";
    child = run("cloudflared", ["tunnel", "--no-autoupdate", "--url", target]);
    pattern = /https:\/\/([a-z0-9-]+)\.trycloudflare\.com/g;
  } else {
    name = "localhost.run (ssh)";
    child = run("ssh", [
      "-o", "StrictHostKeyChecking=accept-new",
      "-o", "ServerAliveInterval=30",
      "-o", "ExitOnForwardFailure=yes",
      "-R", `80:localhost:${VITE_PORT}`,
      "nokey@localhost.run",
    ]);
    // Адрес туннеля: <random>.lhr.life (или <random>.localhost.run у старых аккаунтов).
    // Ссылки из приветствия (admin.localhost.run, localhost.run/docs, twitter) отбрасываются ниже.
    pattern = /https:\/\/([a-z0-9-]+)\.(?:lhr\.life|localhost\.run)(?![\w./-]*\/docs)/g;
  }
  console.log(`→ Поднимаю туннель через ${name}...`);

  const SERVICE_SUBDOMAINS = new Set(["admin", "www", "docs", "api"]);
  const extractUrl = (text) => {
    for (const m of text.matchAll(pattern)) {
      if (m[1] && SERVICE_SUBDOMAINS.has(m[1])) continue;
      return m[0];
    }
    return null;
  };

  return new Promise((resolvePromise, reject) => {
    let found = false;
    let current = null;
    const onData = (chunk) => {
      const text = chunk.toString();
      // Показываем вывод туннеля как есть — так видно, если что-то пошло не так.
      for (const line of text.split("\n")) if (line.trim()) console.log(`  [tunnel] ${line.trim()}`);
      const url = extractUrl(text);
      if (!url || url === current) return;
      current = url;
      if (!found) {
        found = true;
        resolvePromise(url);
      } else {
        // Бесплатный localhost.run время от времени выдаёт новый домен — переставляем кнопку.
        onUrlChanged(url);
      }
    };
    child.stdout.on("data", onData);
    child.stderr.on("data", onData);
    child.on("exit", (code) => {
      if (!found) reject(new Error(`${name} завершился с кодом ${code}, адрес не получен`));
      else {
        console.error(`✗ Туннель ${name} упал (код ${code}). Перезапустите npm run tg.`);
        shutdown(1);
      }
    });
    setTimeout(() => !found && reject(new Error(`${name}: не дождался адреса за 30 секунд`)), 30_000);
  });
}

let shuttingDown = false;
async function shutdown(code = 0) {
  if (shuttingDown) return;
  shuttingDown = true;
  console.log("\n→ Возвращаю кнопку меню по умолчанию...");
  try {
    await setMenuButton({ type: "default" });
  } catch (e) {
    console.error(`  не удалось: ${e.message}`);
  }
  for (const c of children) c.kill("SIGTERM");
  process.exit(code);
}
process.on("SIGINT", () => shutdown(0));
process.on("SIGTERM", () => shutdown(0));

// --- main ---
const me = await telegram("getMe", {});
console.log(`→ Бот: @${me.username}`);

// Запускаем vite напрямую, а не через npx: иначе SIGTERM получит только обёртка,
// а сам vite останется висеть на порту.
const vite = run(resolve(FRONTEND_DIR, "node_modules", ".bin", "vite"), [], {
  cwd: FRONTEND_DIR,
  stdio: ["ignore", "inherit", "inherit"],
});
vite.on("exit", (code) => !shuttingDown && shutdown(code ?? 1));

const webAppButton = (url) => ({ type: "web_app", text: BUTTON_TEXT, web_app: { url } });

async function onUrlChanged(url) {
  try {
    await setMenuButton(webAppButton(url));
    console.log(`↻ Адрес туннеля сменился, кнопка обновлена: ${url}`);
  } catch (e) {
    console.error(`✗ Не удалось обновить кнопку на ${url}: ${e.message}`);
  }
}

let url;
try {
  url = await startTunnel();
} catch (e) {
  console.error(`✗ ${e.message}`);
  await shutdown(1);
}

console.log(`→ Ставлю кнопку меню на ${url} ...`);
try {
  await setMenuButton(webAppButton(url));
} catch (e) {
  console.error(`✗ Не удалось поставить кнопку: ${e.message}`);
  await shutdown(1);
}

console.log(`
✓ Mini App доступен: ${url}
✓ Кнопка меню «${BUTTON_TEXT}» установлена ${chatId ? `в чате ${chatId}` : "для всех пользователей"} бота @${me.username}
  API проксируется на http://127.0.0.1:${PORT || 8080} — бэкенд должен быть запущен (cargo run).

  Откройте чат с @${me.username} и нажмите кнопку «${BUTTON_TEXT}» слева от поля ввода.
  (Если кнопка не появилась — перезайдите в чат.)

  Ctrl+C — остановить и убрать кнопку.
`);
