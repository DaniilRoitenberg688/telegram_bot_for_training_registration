import { useState } from "react";
import { api, ApiError } from "../api/client";
import type { IsoDate, Training, User } from "../api/types";
import { Async, Chip, Empty, Screen } from "../components/ui";
import {
  formatDayMonth,
  formatFullDate,
  formatTime,
  formatWeek,
  formatWeekdayShort,
  upcomingWeeks,
} from "../lib/dates";
import { useAsync } from "../lib/useAsync";
import { alert, confirm, haptic } from "../telegram";

const weeks = upcomingWeeks();

export function Book({ user }: { user: User }) {
  const [weekIndex, setWeekIndex] = useState(0);
  const [selectedDate, setSelectedDate] = useState<IsoDate | null>(null);
  const week = weeks[weekIndex];

  const days = useAsync(async () => {
    const trainings = await api.getDaysWithSlots(week.from, week.to);
    return [...new Set(trainings.map((t) => t.date))].sort();
  }, [week.from, week.to]);

  // Если выбранного дня нет в текущей неделе — показываем первый доступный.
  const availableDates = days.state.status === "success" ? days.state.data : [];
  const activeDate =
    selectedDate && availableDates.includes(selectedDate) ? selectedDate : (availableDates[0] ?? null);

  return (
    <Screen title="Запись на тренировку">
      <section className="section">
        <h2 className="section-title">Неделя</h2>
        <div className="chips">
          {weeks.map((w, i) => (
            <Chip
              key={w.from}
              active={i === weekIndex}
              onClick={() => {
                haptic("selection");
                setWeekIndex(i);
              }}
            >
              {formatWeek(w)}
            </Chip>
          ))}
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">День</h2>
        <Async state={days.state} onRetry={days.reload}>
          {(dates) =>
            dates.length === 0 ? (
              <Empty>На этой неделе свободных тренировок нет</Empty>
            ) : (
              <div className="chips">
                {dates.map((date) => (
                  <Chip
                    key={date}
                    active={date === activeDate}
                    onClick={() => {
                      haptic("selection");
                      setSelectedDate(date);
                    }}
                  >
                    <span className="chip-day">
                      <span className="chip-day-weekday">{formatWeekdayShort(date)}</span>
                      <span>{formatDayMonth(date)}</span>
                    </span>
                  </Chip>
                ))}
              </div>
            )
          }
        </Async>
      </section>

      {activeDate && <Slots key={activeDate} user={user} date={activeDate} />}
    </Screen>
  );
}

function Slots({ user, date }: { user: User; date: IsoDate }) {
  const slots = useAsync(() => api.getFreeSlots(date), [date]);
  const [busyId, setBusyId] = useState<string | null>(null);

  async function book(slot: Training) {
    const ok = await confirm(`Записаться на ${formatFullDate(slot.date)} в ${formatTime(slot.start_time)}?`);
    if (!ok) return;

    setBusyId(slot.id);
    try {
      await api.register(user.id, slot.id);
      haptic("success");
      await alert("✅ Вы успешно записаны на тренировку!");
    } catch (e) {
      haptic("error");
      await alert(bookingErrorMessage(e));
    } finally {
      setBusyId(null);
      slots.reload();
    }
  }

  return (
    <section className="section">
      <h2 className="section-title">Время · {formatFullDate(date)}</h2>
      <Async state={slots.state} onRetry={slots.reload}>
        {(list) => {
          const sorted = [...list].sort((a, b) => a.start_time.localeCompare(b.start_time));
          return sorted.length === 0 ? (
            <Empty>На этот день всё занято</Empty>
          ) : (
            <div className="slots">
              {sorted.map((slot) => (
                <button
                  key={slot.id}
                  className="slot"
                  disabled={busyId !== null}
                  onClick={() => book(slot)}
                >
                  {busyId === slot.id ? "…" : formatTime(slot.start_time)}
                </button>
              ))}
            </div>
          );
        }}
      </Async>
    </section>
  );
}

function bookingErrorMessage(e: unknown): string {
  if (e instanceof ApiError) {
    if (e.status === 403) return "Это время уже заняли. Выберите другое.";
    if (e.status === 404) return "Тренировка не найдена — возможно, её отменили.";
  }
  return "Не удалось записаться. Попробуйте позже.";
}
