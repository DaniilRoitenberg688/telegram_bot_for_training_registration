import { useState } from "react";
import { api } from "../api/client";
import type { IsoDate, TrainerSlot } from "../api/types";
import { Async, Chip, Empty, Screen } from "../components/ui";
import {
  formatDayMonth,
  formatFullDate,
  formatTime,
  formatWeek,
  formatWeekdayShort,
  nowTimeIso,
  todayIso,
  upcomingWeeks,
  weekDaysFromToday,
} from "../lib/dates";
import { useAsync } from "../lib/useAsync";
import { alert, haptic } from "../telegram";

const weeks = upcomingWeeks();

/** Управление расписанием тренера: включение/отключение слотов. */
export function Schedule() {
  const [weekIndex, setWeekIndex] = useState(0);
  const [selectedDate, setSelectedDate] = useState<IsoDate | null>(null);
  const days = weekDaysFromToday(weeks[weekIndex]);
  const activeDate = selectedDate && days.includes(selectedDate) ? selectedDate : days[0];

  return (
    <Screen title="Моё расписание" subtitle="Отключённое время клиенты не видят">
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
        <div className="chips">
          {days.map((date) => (
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
      </section>

      {activeDate && <DaySlots key={activeDate} date={activeDate} />}
    </Screen>
  );
}

function DaySlots({ date }: { date: IsoDate }) {
  const slots = useAsync(async () => {
    const list = await api.getTrainerSlots(date);
    // На сегодня прошедшие часы не показываем — менять их уже бессмысленно
    if (date !== todayIso()) return list;
    const now = nowTimeIso();
    return list.filter((s) => s.start_time >= now);
  }, [date]);

  // Локальные изменения поверх загруженных данных — без перезагрузки и мигания
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});
  const [pending, setPending] = useState<Set<string>>(new Set());
  const isEnabled = (s: TrainerSlot) => overrides[s.id] ?? s.enabled;

  /** Сразу меняет вид слотов, а если сервер ответил ошибкой — возвращает как было. */
  async function setEnabled(targets: TrainerSlot[], enabled: boolean) {
    targets = targets.filter((s) => !pending.has(s.id) && isEnabled(s) !== enabled);
    if (targets.length === 0) return;
    haptic("selection");

    const ids = targets.map((s) => s.id);
    const previous = Object.fromEntries(targets.map((s) => [s.id, isEnabled(s)]));
    setOverrides((o) => ({ ...o, ...Object.fromEntries(ids.map((id) => [id, enabled])) }));
    setPending((p) => new Set([...p, ...ids]));

    const results = await Promise.allSettled(targets.map((s) => api.setTrainingEnabled(s.id, enabled)));
    const failed = targets.filter((_, i) => results[i].status === "rejected");

    setPending((p) => new Set([...p].filter((id) => !ids.includes(id))));
    if (failed.length > 0) {
      setOverrides((o) => ({ ...o, ...Object.fromEntries(failed.map((s) => [s.id, previous[s.id]])) }));
      haptic("error");
      await alert("Не удалось изменить расписание. Проверьте связь и попробуйте ещё раз.");
    }
  }

  return (
    <section className="section">
      <h2 className="section-title">Время · {formatFullDate(date)}</h2>
      <p className="hint schedule-hint">Нажмите на время, чтобы закрыть или открыть его для записи</p>
      <Async state={slots.state} onRetry={slots.reload}>
        {(list) => {
          if (list.length === 0) return <Empty>На этот день тренировок нет</Empty>;
          const allEnabled = list.every(isEnabled);
          return (
            <>
              <div className="slots">
                {list.map((slot, i) => {
                  const enabled = isEnabled(slot);
                  return (
                    <button
                      key={`${slot.id}-${i}`}
                      className={`slot schedule-slot ${enabled ? "" : "schedule-slot-off"} ${
                        slot.full_name ? "schedule-slot-booked" : ""
                      }`}
                      aria-pressed={!enabled}
                      onClick={() => setEnabled([slot], !enabled)}
                    >
                      <span>{formatTime(slot.start_time)}</span>
                      {slot.full_name && <span className="schedule-slot-name">{slot.full_name}</span>}
                    </button>
                  );
                })}
              </div>
              <button
                className={`button button-block ${allEnabled ? "button-danger" : "button-secondary"}`}
                onClick={() => setEnabled(list, !allEnabled)}
              >
                {allEnabled ? "Закрыть весь день" : "Открыть весь день"}
              </button>
            </>
          );
        }}
      </Async>
    </section>
  );
}
