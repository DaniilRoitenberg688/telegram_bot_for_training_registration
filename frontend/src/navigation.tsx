import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import type { IsoDate } from "./api/types";
import { showBackButton } from "./telegram";

// Маленький стековый роутер: экранов мало, а «Назад» должна обрабатывать
// нативная кнопка Telegram, так что полноценный react-router не нужен.
export type Route =
  | { name: "home" }
  | { name: "book" }
  | { name: "my" }
  | { name: "trainer" }
  | { name: "trainerDay"; date: IsoDate }
  | { name: "schedule" };

interface Navigation {
  route: Route;
  canGoBack: boolean;
  push(route: Route): void;
  back(): void;
}

const NavigationContext = createContext<Navigation | null>(null);

export function NavigationProvider({ children }: { children: ReactNode }) {
  const [stack, setStack] = useState<Route[]>([{ name: "home" }]);

  const push = useCallback((route: Route) => setStack((s) => [...s, route]), []);
  const back = useCallback(() => setStack((s) => (s.length > 1 ? s.slice(0, -1) : s)), []);

  const canGoBack = stack.length > 1;
  useEffect(() => {
    if (canGoBack) return showBackButton(back);
  }, [canGoBack, back]);

  // Фигурные скобки обязательны: в новых браузерах scrollTo возвращает Promise,
  // и React принял бы его за функцию очистки — приложение падает при переходе между экранами.
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [stack.length]);

  const value = useMemo(
    () => ({ route: stack[stack.length - 1], canGoBack, push, back }),
    [stack, canGoBack, push, back],
  );
  return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
}

/** То же, что useNavigation, но вне NavigationProvider возвращает null (для общих компонентов). */
export function useOptionalNavigation(): Navigation | null {
  return useContext(NavigationContext);
}

export function useNavigation(): Navigation {
  const ctx = useContext(NavigationContext);
  if (!ctx) throw new Error("useNavigation must be used inside NavigationProvider");
  return ctx;
}
