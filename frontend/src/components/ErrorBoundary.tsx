import { Component, type ErrorInfo, type ReactNode } from "react";

/** Ловит ошибки рендера: вместо пустого белого экрана показывает сообщение и кнопку перезагрузки. */
export class ErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("App crashed:", error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <main className="screen">
        <div className="center message">
          <p>Что-то пошло не так</p>
          <p className="hint">{this.state.error.message}</p>
          <button className="button" onClick={() => window.location.reload()}>
            Перезагрузить
          </button>
        </div>
      </main>
    );
  }
}
