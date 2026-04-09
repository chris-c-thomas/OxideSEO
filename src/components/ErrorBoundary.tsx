/**
 * React Error Boundary for graceful crash recovery.
 *
 * Catches uncaught rendering exceptions and displays a fallback UI
 * instead of a blank white screen. Provides a retry button to
 * attempt re-rendering the failed subtree.
 */

import { Component, type ErrorInfo, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { AlertCircle, RotateCcw } from "lucide-react";

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
          <div className="bg-danger/10 rounded-[var(--radius-lg)] p-3">
            <AlertCircle className="text-danger size-6" strokeWidth={1.75} />
          </div>
          <div className="flex flex-col items-center gap-1 text-center">
            <h2 className="text-fg-default text-sm font-medium">Something went wrong</h2>
            <p className="text-fg-muted max-w-[360px] text-xs">
              An unexpected error occurred. Try again, or restart the application if the
              problem persists.
            </p>
            {this.state.error && (
              <pre className="bg-bg-muted text-fg-subtle mt-2 max-w-[400px] truncate rounded-[var(--radius-sm)] px-3 py-1.5 font-mono text-[0.6875rem]">
                {this.state.error.message}
              </pre>
            )}
          </div>
          <Button size="sm" variant="outline" onClick={this.handleReset}>
            <RotateCcw className="size-3.5" strokeWidth={1.75} />
            Try Again
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
