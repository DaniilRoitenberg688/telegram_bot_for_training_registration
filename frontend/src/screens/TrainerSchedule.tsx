import { api } from "../api/client";
import { Async, Empty, Screen } from "../components/ui";
import { formatFullDate, formatWeek, isInWeek, upcomingWeeks } from "../lib/dates";
import { useAsync } from "../lib/useAsync";
import { useNavigation } from "../navigation";

const weeks = upcomingWeeks();

export function TrainerSchedule() {
  const { push } = useNavigation();
  // Один запрос на все 4 недели вместо /trainer/weeks: группируем по неделям на клиенте.
  const days = useAsync(async () => {
    const trainings = await api.getTrainerDays(weeks[0].from, weeks[weeks.length - 1].to);
    return [...new Set(trainings.map((t) => t.date))].sort();
  }, []);

  return (
    <Screen title="Записи клиентов">
      <Async state={days.state} onRetry={days.reload}>
        {(dates) => {
          const groups = weeks
            .map((w) => ({ week: w, dates: dates.filter((d) => isInWeek(d, w)) }))
            .filter((g) => g.dates.length > 0);

          if (groups.length === 0) return <Empty>Пока никто не записан</Empty>;

          return groups.map(({ week, dates }) => (
            <section key={week.from} className="section">
              <h2 className="section-title">{formatWeek(week)}</h2>
              <div className="list">
                {dates.map((date) => (
                  <button
                    key={date}
                    className="list-item"
                    onClick={() => push({ name: "trainerDay", date })}
                  >
                    <span className="list-item-body">
                      <span className="list-item-title">{formatFullDate(date)}</span>
                    </span>
                    <span className="chevron">›</span>
                  </button>
                ))}
              </div>
            </section>
          ));
        }}
      </Async>
    </Screen>
  );
}
