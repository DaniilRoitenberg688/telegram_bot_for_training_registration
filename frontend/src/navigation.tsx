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
  | { name: "trainerDay"; date: IsoDate };

interface Navigation {
  route: Route;
  push(route: Route): void;
  back(): void;
}

const NavigationContext = createContext<Navigation | null>(null);

export function NavigationProvider({ children }: { children: ReactNode }) {
  const [stack, setStack] = useState<Route[]>([{ name: "home" }]);

  const push = useCallback((route: Route) => setStack((s) => [...s, route]), []);
  const back = useCallback(() => setStack((s) => (s.length > 1 ? s.slice(0, -1) : s)), []);

  const canGoBack = stack.length > 1;
  useEffect(() => (canGoBack ? showBackButton(back) : undefined), [canGoBack, back]);

  useEffect(() => window.scrollTo(0, 0), [stack.length]);

  const value = useMemo(() => ({ route: stack[stack.length - 1], push, back }), [stack, push, back]);
  return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
}

export function useNavigation(): Navigation {
  const ctx = useContext(NavigationContext);
  if (!ctx) throw new Error("useNavigation must be used inside NavigationProvider");
  return ctx;
}
