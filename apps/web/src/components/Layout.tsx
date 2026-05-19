import { Outlet, Link, useLocation } from 'react-router-dom'
import { Shield, Map, BarChart3, FileText, Menu, X } from 'lucide-react'
import { useState } from 'react'

export default function Layout() {
  const [menuOpen, setMenuOpen] = useState(false)
  const location = useLocation()

  const navItems = [
    { path: '/', label: 'Home', icon: Shield },
    { path: '/report', label: 'Report Issue', icon: FileText },
    { path: '/map', label: 'Live Map', icon: Map },
    { path: '/dashboard', label: 'Analytics', icon: BarChart3 },
  ]

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="sticky top-0 z-50 glass-panel border-b border-slate-700/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link to="/" className="flex items-center space-x-3">
              <Shield className="w-8 h-8 text-emerald-400" />
              <span className="text-xl font-bold gradient-text">Civic Sentinel</span>
            </Link>

            {/* Desktop Nav */}
            <nav className="hidden md:flex items-center space-x-1">
              {navItems.map((item) => (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-all ${
                    location.pathname === item.path
                      ? 'bg-emerald-500/10 text-emerald-400'
                      : 'text-slate-300 hover:text-white hover:bg-slate-700/50'
                  }`}
                >
                  <item.icon className="w-4 h-4" />
                  <span className="text-sm font-medium">{item.label}</span>
                </Link>
              ))}
            </nav>

            {/* Mobile Menu Button */}
            <button
              className="md:hidden p-2 text-slate-300 hover:text-white"
              onClick={() => setMenuOpen(!menuOpen)}
            >
              {menuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </button>
          </div>
        </div>

        {/* Mobile Nav */}
        {menuOpen && (
          <div className="md:hidden border-t border-slate-700/50 bg-slate-900/95">
            {navItems.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center space-x-3 px-4 py-3 ${
                  location.pathname === item.path
                    ? 'text-emerald-400 bg-emerald-500/10'
                    : 'text-slate-300'
                }`}
                onClick={() => setMenuOpen(false)}
              >
                <item.icon className="w-5 h-5" />
                <span>{item.label}</span>
              </Link>
            ))}
          </div>
        )}
      </header>

      {/* Main Content */}
      <main className="flex-1">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-700/50 bg-slate-900/50 py-8">
        <div className="max-w-7xl mx-auto px-4 text-center text-slate-400">
          <p className="text-sm">
            Built with ❤️ by{' '}
            <a
              href="https://github.com/FirstPrinciples-Sun"
              className="text-emerald-400 hover:text-emerald-300"
              target="_blank"
              rel="noopener noreferrer"
            >
              FirstPrinciples-Sun
            </a>{' '}
            and the open source community.
          </p>
          <p className="text-xs mt-2 text-slate-500">
            Civic Sentinel is open source under MIT License.
          </p>
        </div>
      </footer>
    </div>
  )
}
