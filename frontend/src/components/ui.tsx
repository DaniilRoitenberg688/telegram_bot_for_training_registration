import type { ReactNode } from "react";
import type { AsyncState } from "../lib/useAsync";
import { useOptionalNavigation } from "../navigation";

export function Screen({ title, subtitle, children }: { title: string; subtitle?: string; children: ReactNode }) {
  const nav = useOptionalNavigation();
  return (
    <main className="screen">
      <header className="screen-header">
        {nav?.canGoBack && (
          <button className="back-link" onClick={nav.back}>
            ‹ Назад
          </button>
        )}
        <h1>{title}</h1>
        {subtitle && <p className="hint">{subtitle}</p>}
      </header>
      {children}
    </main>
  );
}

export function Spinner() {
  return (
    <div className="center">
      <div className="spinner" aria-label="Загрузка" />
    </div>
  );
}

export function ErrorView({ error, onRetry }: { error: Error; onRetry?: () => void }) {
  return (
    <div className="center message">
      <p>Не удалось загрузить данные</p>
      <p className="hint">{error.message}</p>
      {onRetry && (
        <button className="button button-secondary" onClick={onRetry}>
          Повторить
        </button>
      )}
    </div>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="center message hint">{children}</div>;
}

/** Рисует спиннер / ошибку / данные в зависимости от состояния загрузки. */
export function Async<T>({
  state,
  onRetry,
  children,
}: {
  state: AsyncState<T>;
  onRetry?: () => void;
  children: (data: T) => ReactNode;
}) {
  if (state.status === "loading") return <Spinner />;
  if (state.status === "error") return <ErrorView error={state.error} onRetry={onRetry} />;
  return <>{children(state.data)}</>;
}

export function Chip({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button className={`chip ${active ? "chip-active" : ""}`} onClick={onClick}>
      {children}
    </button>
  );
}
