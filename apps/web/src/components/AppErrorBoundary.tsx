import React from 'react'

type State = {
  hasError: boolean
}

export default class AppErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  public state: State = { hasError: false }

  public static getDerivedStateFromError(): State {
    return { hasError: true }
  }

  public componentDidCatch(error: Error) {
    // Keep console visibility for developer diagnostics.
    console.error('Unhandled UI error:', error)
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center px-4">
          <div className="max-w-md w-full rounded-xl border border-slate-300 bg-white p-6 text-center shadow-sm">
            <h1 className="text-xl font-semibold text-slate-900">Something went wrong</h1>
            <p className="mt-2 text-sm text-slate-600">
              Please refresh this page. If the issue persists, clear browser storage and try again.
            </p>
            <button
              className="mt-5 rounded-lg bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700"
              onClick={() => window.location.reload()}
            >
              Reload
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
