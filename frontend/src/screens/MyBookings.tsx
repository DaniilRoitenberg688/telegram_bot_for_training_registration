import { useState } from "react";
import { api } from "../api/client";
import type { Training, User } from "../api/types";
import { Async, Empty, Screen } from "../components/ui";
import { formatFullDate, formatTime } from "../lib/dates";
import { useAsync } from "../lib/useAsync";
import { useNavigation } from "../navigation";
import { alert, confirm, haptic } from "../telegram";

export function MyBookings({ user }: { user: User }) {
  const { push } = useNavigation();
  const bookings = useAsync(() => api.getMyTrainings(user.id), [user.id]);
  const [busyId, setBusyId] = useState<string | null>(null);

  async function cancel(t: Training) {
    const ok = await confirm(`Отменить запись: ${formatFullDate(t.date)}, ${formatTime(t.start_time)}?`);
    if (!ok) return;

    setBusyId(t.id);
    try {
      await api.unregister(user.id, t.id);
      haptic("success");
    } catch {
      haptic("error");
      await alert("Не удалось отменить запись. Попробуйте позже.");
    } finally {
      setBusyId(null);
      bookings.reload();
    }
  }

  return (
    <Screen title="Мои записи">
      <Async state={bookings.state} onRetry={bookings.reload}>
        {(list) =>
          list.length === 0 ? (
            <Empty>
              <p>У вас пока нет записей</p>
              <button className="button" onClick={() => push({ name: "book" })}>
                Записаться
              </button>
            </Empty>
          ) : (
            <div className="list">
              {list.map((t) => (
                <div key={t.id} className="list-item">
                  <span className="list-item-icon">🥊</span>
                  <span className="list-item-body">
                    <span className="list-item-title">{formatFullDate(t.date)}</span>
                    <span className="hint">
                      {formatTime(t.start_time)} — {formatTime(t.end_time)}
                    </span>
                  </span>
                  <button
                    className="button button-danger button-small"
                    disabled={busyId !== null}
                    onClick={() => cancel(t)}
                  >
                    {busyId === t.id ? "…" : "Отменить"}
                  </button>
                </div>
              ))}
            </div>
          )
        }
      </Async>
    </Screen>
  );
}
