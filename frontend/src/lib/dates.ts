import type { IsoDate, IsoTime } from "../api/types";

// Бэкенд считает всё по московскому времени, поэтому и «сегодня» берём по Москве,
// а не по часовому поясу телефона. Внутри работаем с датами в UTC, чтобы не ловить сдвиги.
const TIMEZONE = "Europe/Moscow";

const DAY_MS = 24 * 60 * 60 * 1000;

function toDate(iso: IsoDate): Date {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(Date.UTC(y, m - 1, d));
}

function toIso(date: Date): IsoDate {
  return date.toISOString().slice(0, 10);
}

export function todayIso(): IsoDate {
  // en-CA форматирует как YYYY-MM-DD
  return new Intl.DateTimeFormat("en-CA", { timeZone: TIMEZONE }).format(new Date());
}

export function addDays(iso: IsoDate, days: number): IsoDate {
  return toIso(new Date(toDate(iso).getTime() + days * DAY_MS));
}

export interface Week {
  from: IsoDate; // понедельник
  to: IsoDate; // воскресенье
}

/** Текущая и ещё `count - 1` недель вперёд — как get_weeks() в боте. */
export function upcomingWeeks(count = 4): Week[] {
  const today = todayIso();
  const daysFromMonday = (toDate(today).getUTCDay() + 6) % 7;
  const monday = addDays(today, -daysFromMonday);
  return Array.from({ length: count }, (_, i) => ({
    from: addDays(monday, i * 7),
    to: addDays(monday, i * 7 + 6),
  }));
}

const fmt = (options: Intl.DateTimeFormatOptions) =>
  new Intl.DateTimeFormat("ru-RU", { timeZone: "UTC", ...options });

const dayMonth = fmt({ day: "numeric", month: "long" });
const shortDayMonth = fmt({ day: "2-digit", month: "2-digit" });
const weekdayShort = fmt({ weekday: "short" });
const weekdayLong = fmt({ weekday: "long" });

/** «9 ноября» */
export const formatDayMonth = (iso: IsoDate) => dayMonth.format(toDate(iso));
/** «09.11» */
export const formatShortDate = (iso: IsoDate) => shortDayMonth.format(toDate(iso));
/** «пн» */
export const formatWeekdayShort = (iso: IsoDate) => weekdayShort.format(toDate(iso));
/** «Понедельник» */
export const formatWeekday = (iso: IsoDate) => {
  const s = weekdayLong.format(toDate(iso));
  return s.charAt(0).toUpperCase() + s.slice(1);
};
/** «Понедельник, 9 ноября» */
export const formatFullDate = (iso: IsoDate) => `${formatWeekday(iso)}, ${formatDayMonth(iso)}`;

/** «07:00:00» -> «07:00» */
export const formatTime = (time: IsoTime) => time.slice(0, 5);

export const formatWeek = (w: Week) => `${formatShortDate(w.from)} — ${formatShortDate(w.to)}`;

export const isInWeek = (iso: IsoDate, w: Week) => iso >= w.from && iso <= w.to;
