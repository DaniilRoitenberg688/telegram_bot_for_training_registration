// Минимальные типы для той части Telegram WebApp API, которой мы пользуемся.
// Полная документация: https://core.telegram.org/bots/webapps

interface WebAppUser {
  id: number;
  first_name: string;
  last_name?: string;
  username?: string;
}

interface BackButton {
  show(): void;
  hide(): void;
  onClick(cb: () => void): void;
  offClick(cb: () => void): void;
}

interface WebApp {
  initData: string;
  initDataUnsafe: { user?: WebAppUser };
  version: string;
  colorScheme: "light" | "dark";
  BackButton: BackButton;
  HapticFeedback: {
    notificationOccurred(type: "error" | "success" | "warning"): void;
    selectionChanged(): void;
  };
  ready(): void;
  expand(): void;
  close(): void;
  isVersionAtLeast(version: string): boolean;
  showAlert(message: string, cb?: () => void): void;
  showConfirm(message: string, cb: (ok: boolean) => void): void;
  openTelegramLink(url: string): void;
}

declare global {
  interface Window {
    Telegram?: { WebApp: WebApp };
  }
}

const webApp: WebApp | undefined = window.Telegram?.WebApp;

/** Открыто ли приложение внутри Telegram (а не просто в браузере). */
export const insideTelegram = Boolean(webApp?.initData);

export function initTelegram() {
  webApp?.ready();
  webApp?.expand();
}

/** id текущего пользователя Telegram. Вне Telegram берётся VITE_DEV_USER_ID. */
export function getUserId(): string | null {
  const id = webApp?.initDataUnsafe.user?.id;
  if (id) return String(id);
  return import.meta.env.VITE_DEV_USER_ID || null;
}

/** Подписанная строка initData — её бэкенд должен проверять для авторизации. */
export function getInitData(): string {
  return webApp?.initData ?? "";
}

const popupsSupported = () => insideTelegram && webApp!.isVersionAtLeast("6.2");

export function confirm(message: string): Promise<boolean> {
  if (popupsSupported()) {
    return new Promise((resolve) => webApp!.showConfirm(message, resolve));
  }
  return Promise.resolve(window.confirm(message));
}

export function alert(message: string): Promise<void> {
  if (popupsSupported()) {
    return new Promise((resolve) => webApp!.showAlert(message, () => resolve()));
  }
  window.alert(message);
  return Promise.resolve();
}

export function haptic(type: "error" | "success" | "warning" | "selection") {
  if (!insideTelegram || !webApp!.isVersionAtLeast("6.1")) return;
  if (type === "selection") webApp!.HapticFeedback.selectionChanged();
  else webApp!.HapticFeedback.notificationOccurred(type);
}

export function openTelegramProfile(username: string) {
  const url = `https://t.me/${username}`;
  if (insideTelegram) webApp!.openTelegramLink(url);
  else window.open(url, "_blank");
}

export function closeApp() {
  webApp?.close();
}

/** Нативная кнопка «Назад» в шапке Telegram. Возвращает функцию отписки. */
export function showBackButton(onClick: () => void): () => void {
  if (!insideTelegram || !webApp!.isVersionAtLeast("6.1")) return () => {};
  webApp!.BackButton.onClick(onClick);
  webApp!.BackButton.show();
  return () => {
    webApp!.BackButton.offClick(onClick);
    webApp!.BackButton.hide();
  };
}
