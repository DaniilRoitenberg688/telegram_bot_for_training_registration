import type { User } from "../api/types";
import { Screen } from "../components/ui";
import { useNavigation, type Route } from "../navigation";

interface MenuItem {
  icon: string;
  title: string;
  description: string;
  route: Route;
}

const clientMenu: MenuItem[] = [
  { icon: "🥊", title: "Записаться", description: "Выбрать день и время тренировки", route: { name: "book" } },
  { icon: "📋", title: "Мои записи", description: "Посмотреть или отменить запись", route: { name: "my" } },
];

const trainerMenu: MenuItem[] = [
  { icon: "📅", title: "Записи клиентов", description: "Кто и когда записан", route: { name: "trainer" } },
  { icon: "🗓", title: "Моё расписание", description: "Включить или отключить тренировки", route: { name: "schedule" } },
];

export function Home({ user }: { user: User }) {
  const { push } = useNavigation();
  const menu = user.is_trainer ? trainerMenu : clientMenu;
  const greeting = user.is_trainer ? "Здравствуйте, тренер!" : `Здравствуйте, ${user.full_name}!`;

  return (
    <Screen title={greeting} subtitle="Чем могу помочь?">
      <nav className="list">
        {menu.map((item) => (
          <button key={item.title} className="list-item" onClick={() => push(item.route)}>
            <span className="list-item-icon">{item.icon}</span>
            <span className="list-item-body">
              <span className="list-item-title">{item.title}</span>
              <span className="hint">{item.description}</span>
            </span>
            <span className="chevron">›</span>
          </button>
        ))}
      </nav>
    </Screen>
  );
}
