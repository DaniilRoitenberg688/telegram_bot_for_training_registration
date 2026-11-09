import { api } from "../api/client";
import type { IsoDate } from "../api/types";
import { Async, Empty, Screen } from "../components/ui";
import { formatFullDate, formatTime } from "../lib/dates";
import { useAsync } from "../lib/useAsync";
import { openTelegramProfile } from "../telegram";

export function TrainerDay({ date }: { date: IsoDate }) {
  const registrations = useAsync(async () => {
    const list = await api.getTrainerDayRegistrations(date);
    return [...list].sort((a, b) => a.start_time.localeCompare(b.start_time));
  }, [date]);

  return (
    <Screen title={formatFullDate(date)}>
      <Async state={registrations.state} onRetry={registrations.reload}>
        {(list) =>
          list.length === 0 ? (
            <Empty>На этот день записей нет</Empty>
          ) : (
            <div className="list">
              {list.map((r) => (
                <div key={`${r.start_time}-${r.username}-${r.full_name}`} className="list-item">
                  <span className="time-badge">{formatTime(r.start_time)}</span>
                  <span className="list-item-body">
                    <span className="list-item-title">{r.full_name}</span>
                    {r.username ? <span className="hint">@{r.username}</span> : <span className="hint">без username</span>}
                  </span>
                  {r.username && (
                    <button
                      className="button button-secondary button-small"
                      onClick={() => openTelegramProfile(r.username)}
                    >
                      Написать
                    </button>
                  )}
                </div>
              ))}
            </div>
          )
        }
      </Async>
    </Screen>
  );
}
