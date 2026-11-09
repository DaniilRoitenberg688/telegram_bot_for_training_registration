import { api, ApiError } from "./api/client";
import type { User } from "./api/types";
import { Async, Screen } from "./components/ui";
import { useAsync } from "./lib/useAsync";
import { NavigationProvider, useNavigation } from "./navigation";
import { Book } from "./screens/Book";
import { Home } from "./screens/Home";
import { MyBookings } from "./screens/MyBookings";
import { TrainerDay } from "./screens/TrainerDay";
import { TrainerSchedule } from "./screens/TrainerSchedule";
import { closeApp, getUserId, insideTelegram } from "./telegram";

/** null — пользователь не зарегистрирован в боте (GET /user/{id} вернул 404). */
async function loadUser(id: string): Promise<User | null> {
  try {
    return await api.getUser(id);
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) return null;
    throw e;
  }
}

export function App() {
  const userId = getUserId();
  if (!userId) return <OpenInTelegram />;
  return <Authorized userId={userId} />;
}

function Authorized({ userId }: { userId: string }) {
  const user = useAsync(() => loadUser(userId), [userId]);
  return (
    <Async state={user.state} onRetry={user.reload}>
      {(u) =>
        u ? (
          <NavigationProvider>
            <Router user={u} />
          </NavigationProvider>
        ) : (
          <NotRegistered />
        )
      }
    </Async>
  );
}

function Router({ user }: { user: User }) {
  const { route } = useNavigation();
  switch (route.name) {
    case "home":
      return <Home user={user} />;
    case "book":
      return <Book user={user} />;
    case "my":
      return <MyBookings user={user} />;
    case "trainer":
      return <TrainerSchedule />;
    case "trainerDay":
      return <TrainerDay date={route.date} />;
  }
}

function NotRegistered() {
  return (
    <Screen title="Вы ещё не зарегистрированы">
      <div className="center message">
        <p>
          Вернитесь в чат с ботом, отправьте команду <b>/start</b> и введите своё ФИО. После этого
          откройте приложение снова.
        </p>
        {insideTelegram && (
          <button className="button" onClick={closeApp}>
            Вернуться в чат
          </button>
        )}
      </div>
    </Screen>
  );
}

function OpenInTelegram() {
  return (
    <Screen title="Откройте приложение в Telegram">
      <div className="center message hint">
        Это мини-приложение работает внутри Telegram. Для разработки в браузере задайте
        VITE_DEV_USER_ID в frontend/.env.
      </div>
    </Screen>
  );
}
