// Типы повторяют Rust-модели из src/models.rs (сериализация serde по умолчанию).

/** Дата в формате YYYY-MM-DD (chrono::NaiveDate). */
export type IsoDate = string;
/** Время в формате HH:MM:SS (chrono::NaiveTime). */
export type IsoTime = string;

export interface User {
  id: string;
  full_name: string;
  username: string;
  first_name: string;
  last_name: string;
  is_trainer: boolean;
}

export interface Training {
  id: string;
  date: IsoDate;
  start_time: IsoTime;
  end_time: IsoTime;
  capacity: number;
  enabled: boolean;
}

export interface RegistrationFullInfo {
  date: IsoDate;
  start_time: IsoTime;
  end_time: IsoTime;
  full_name: string;
  username: string;
}

/** Слот расписания для тренера (GET /trainer/slots): включая выключенные и занятые. */
export interface TrainerSlot {
  id: string;
  date: IsoDate;
  start_time: IsoTime;
  end_time: IsoTime;
  enabled: boolean;
  user_id: string | null;
  full_name: string | null;
  username: string | null;
}
