import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div style={{ padding: 24, color: "#f87171" }}>
          <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>
            Something went wrong
          </h3>
          <pre style={{ fontSize: 12, whiteSpace: "pre-wrap", color: "#a1a1aa" }}>
            {this.state.error?.message}
          </pre>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            style={{
              marginTop: 12,
              padding: "6px 12px",
              borderRadius: 6,
              background: "#166534",
              color: "#fff",
              border: "none",
              cursor: "pointer",
              fontSize: 13,
            }}
          >
            Try again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
