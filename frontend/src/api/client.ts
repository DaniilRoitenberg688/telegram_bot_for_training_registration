import { getInitData } from "../telegram";
import type { IsoDate, RegistrationFullInfo, TrainerSlot, Training, User } from "./types";

const BASE_URL = (import.meta.env.VITE_API_URL || "/api").replace(/\/$/, "");

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  // Бэкенд проверяет подпись initData (src/api/auth.rs) и берёт из неё id пользователя.
  const initData = getInitData();
  if (initData) {
    headers.set("Authorization", `tma ${initData}`);
  } else if (import.meta.env.VITE_DEV_USER_ID) {
    // Разработка в браузере: работает, только если на бэке API_AUTH_DISABLED=true.
    headers.set("X-Dev-User-Id", import.meta.env.VITE_DEV_USER_ID);
  }
  if (init.body) headers.set("Content-Type", "application/json");

  let res: Response;
  try {
    res = await fetch(`${BASE_URL}${path}`, { ...init, headers });
  } catch {
    throw new ApiError(0, "Нет связи с сервером");
  }

  if (!res.ok) {
    // ApiError на бэке отдаёт {"error": "..."}
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.error ?? `HTTP ${res.status}`);
  }
  // Часть эндпоинтов (201, 204, PUT /training/{id} -> 200) отвечает пустым телом
  const text = await res.text();
  return (text ? JSON.parse(text) : undefined) as T;
}

const query = (params: Record<string, string>) => new URLSearchParams(params).toString();
const json = (body: unknown) => JSON.stringify(body);

export const api = {
  /** GET /user/{id} — 404, если пользователь не прошёл /start в боте. */
  getUser: (id: string) => request<User>(`/user/${encodeURIComponent(id)}`),

  /** GET /days — по одной тренировке на каждый день с доступными слотами (используем только date). */
  getDaysWithSlots: (from: IsoDate, to: IsoDate) =>
    request<Training[]>(`/days?${query({ from, to })}`),

  /** GET /trainings — свободные слоты на дату (уже прошедшие отфильтрованы бэком). */
  getFreeSlots: (date: IsoDate) => request<Training[]>(`/trainings?${query({ date })}`),

  /** GET /registrations/{user_id} — будущие записи пользователя. */
  getMyTrainings: (userId: string) =>
    request<Training[]>(`/registrations/${encodeURIComponent(userId)}`),

  register: (userId: string, trainingId: string) =>
    request<void>("/registrations", {
      method: "POST",
      body: json({ user_id: userId, training_id: trainingId }),
    }),

  unregister: (userId: string, trainingId: string) =>
    request<void>("/registrations", {
      method: "DELETE",
      body: json({ user_id: userId, training_id: trainingId }),
    }),

  /** GET /trainer/days — дни, в которых есть записи. */
  getTrainerDays: (from: IsoDate, to: IsoDate) =>
    request<Training[]>(`/trainer/days?${query({ from, to })}`),

  /** GET /trainer/trainings — кто записан на конкретный день. */
  getTrainerDayRegistrations: (date: IsoDate) =>
    request<RegistrationFullInfo[]>(`/trainer/trainings?${query({ date })}`),

  /** GET /trainer/slots — все слоты дня (включённые, выключенные, занятые). Только тренер. */
  getTrainerSlots: (date: IsoDate) => request<TrainerSlot[]>(`/trainer/slots?${query({ date })}`),

  /** PUT /training/{id} — включить/выключить слот. Только тренер. */
  setTrainingEnabled: (id: string, enabled: boolean) =>
    request<void>(`/training/${encodeURIComponent(id)}`, {
      method: "PUT",
      body: json({ enabled }),
    }),
};
