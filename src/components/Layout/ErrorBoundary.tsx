import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertOctagon, RefreshCw } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * Top-level error boundary that catches unhandled React render errors
 * and displays a recovery screen instead of a blank white page.
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log to console for debugging — no external telemetry (Constitution III)
    console.error("[ErrorBoundary] Uncaught error:", error);
    console.error("[ErrorBoundary] Component stack:", errorInfo.componentStack);
  }

  private handleReload = () => {
    window.location.reload();
  };

  private handleRecover = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <div className="flex h-screen w-screen items-center justify-center bg-background p-8">
        <div className="flex max-w-md flex-col items-center gap-6 text-center">
          <AlertOctagon className="h-16 w-16 text-destructive" />

          <div className="flex flex-col gap-2">
            <h1 className="text-xl font-bold text-foreground">
              Something went wrong
            </h1>
            <p className="text-sm text-muted-foreground">
              An unexpected error occurred. You can try recovering or reload the
              application.
            </p>
          </div>

          {this.state.error && (
            <div className="w-full rounded-md border border-border bg-muted/50 p-3 text-left">
              <p className="font-mono text-xs text-muted-foreground break-all">
                {this.state.error.message}
              </p>
            </div>
          )}

          <div className="flex gap-3">
            <button
              type="button"
              onClick={this.handleRecover}
              className="inline-flex items-center gap-2 rounded-md border border-border bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
            >
              Try Again
            </button>
            <button
              type="button"
              onClick={this.handleReload}
              className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              <RefreshCw className="h-4 w-4" />
              Reload App
            </button>
          </div>
        </div>
      </div>
    );
  }
}
