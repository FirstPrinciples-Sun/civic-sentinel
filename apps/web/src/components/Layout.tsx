import { Outlet, Link, useLocation, useNavigate } from 'react-router-dom'
import { Shield, Map, BarChart3, FileText, Menu, X, LogIn, LogOut, User, ShieldCheck } from 'lucide-react'
import { useState } from 'react'
import { useAuth } from '../context/AuthContext'
import { useI18n } from '../context/LanguageContext'

function roleBadge(role: string, t: (key: string) => string) {
  switch (role) {
    case 'admin': return { label: t('layout.role.admin'), className: 'bg-rose-500/20 text-rose-400 border-rose-500/30' }
    case 'responder': return { label: t('layout.role.responder'), className: 'bg-amber-500/20 text-amber-400 border-amber-500/30' }
    case 'reporter': return { label: t('layout.role.reporter'), className: 'bg-sky-500/20 text-sky-400 border-sky-500/30' }
    default: return { label: t('layout.role.viewer'), className: 'bg-slate-500/20 text-slate-400 border-slate-500/30' }
  }
}

export default function Layout() {
  const [menuOpen, setMenuOpen] = useState(false)
  const location = useLocation()
  const navigate = useNavigate()
  const { user, isAuthenticated, logout } = useAuth()
  const { language, setLanguage, languages, t } = useI18n()

  const navItems = [
    { path: '/', label: t('layout.nav.home'), icon: Shield },
    { path: '/report', label: t('layout.nav.report'), icon: FileText },
    { path: '/map', label: t('layout.nav.map'), icon: Map },
    { path: '/dashboard', label: t('layout.nav.analytics'), icon: BarChart3 },
  ]
  const canAccessAdminQueue = user?.role === 'admin' || user?.role === 'responder'
  const navItemsWithAdmin = canAccessAdminQueue
    ? [...navItems, { path: '/admin', label: t('layout.nav.admin'), icon: ShieldCheck }]
    : navItems

  const handleLogout = async () => {
    await logout()
    navigate('/')
  }

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="sticky top-0 z-50 glass-panel border-b border-slate-700/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <Link to="/" className="flex items-center space-x-3">
              <Shield className="w-8 h-8 text-sky-300" />
              <span className="text-xl font-bold gradient-text">Civic Sentinel</span>
            </Link>

            {/* Desktop Nav */}
            <nav className="hidden md:flex items-center space-x-1">
              {navItemsWithAdmin.map((item) => (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-all ${
                    location.pathname === item.path
                      ? 'bg-sky-500/10 text-sky-300'
                      : 'text-slate-300 hover:text-white hover:bg-slate-700/50'
                  }`}
                >
                  <item.icon className="w-4 h-4" />
                  <span className="text-sm font-medium">{item.label}</span>
                </Link>
              ))}
            </nav>

            {/* Auth Buttons */}
            <div className="flex items-center gap-3">
              <div className="hidden md:flex items-center gap-2">
                <label className="text-xs text-slate-400">{t('layout.lang.label')}</label>
                <select
                  value={language}
                  onChange={(event) => setLanguage(event.target.value as typeof language)}
                  className="bg-slate-800 border border-slate-700 text-slate-200 text-xs rounded px-2 py-1 focus:outline-none"
                >
                  {languages.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.label}
                    </option>
                  ))}
                </select>
              </div>
              {isAuthenticated && user ? (
                <div className="hidden md:flex items-center gap-3">
                  <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-800 border border-slate-700">
                    <User className="w-4 h-4 text-slate-400" />
                    <span className="text-sm text-slate-300">{user.name || user.email}</span>
                    {(() => {
                      const badge = roleBadge(user.role, t)
                      return <span className={`text-xs px-1.5 py-0.5 rounded border ${badge.className}`}>{badge.label}</span>
                    })()}
                  </div>
                  <button
                    onClick={handleLogout}
                    className="flex items-center gap-2 px-3 py-1.5 text-sm text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all"
                  >
                    <LogOut className="w-4 h-4" />
                    <span>{t('layout.auth.logout')}</span>
                  </button>
                </div>
              ) : (
                <div className="hidden md:flex items-center gap-2">
                  <Link
                    to="/login"
                    className="px-4 py-2 text-sm text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all flex items-center gap-2"
                  >
                    <LogIn className="w-4 h-4" /> {t('layout.auth.signIn')}
                  </Link>
                  <Link
                    to="/register"
                    className="px-4 py-2 text-sm bg-sky-700 hover:bg-sky-600 text-white rounded-lg transition-all"
                  >
                    {t('layout.auth.getStarted')}
                  </Link>
                </div>
              )}
              {/* Mobile Menu Button */}
              <button
                className="md:hidden p-2 text-slate-300 hover:text-white"
                onClick={() => setMenuOpen(!menuOpen)}
              >
                {menuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
              </button>
            </div>
          </div>
        </div>

        {/* Mobile Nav */}
        {menuOpen && (
          <div className="md:hidden border-t border-slate-700/50 bg-slate-900/95">
            {navItemsWithAdmin.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center space-x-3 px-4 py-3 ${
                  location.pathname === item.path
                    ? 'text-sky-300 bg-sky-500/10'
                    : 'text-slate-300'
                }`}
                onClick={() => setMenuOpen(false)}
              >
                <item.icon className="w-5 h-5" />
                <span>{item.label}</span>
              </Link>
            ))}
            {isAuthenticated ? (
              <button
                onClick={() => { handleLogout(); setMenuOpen(false) }}
                className="flex items-center space-x-3 px-4 py-3 text-slate-300 w-full"
              >
                <LogOut className="w-5 h-5" />
                <span>{t('layout.auth.logout')} ({user?.name || user?.email})</span>
              </button>
            ) : (
              <>
                <Link to="/login" className="flex items-center space-x-3 px-4 py-3 text-slate-300" onClick={() => setMenuOpen(false)}>
                  <LogIn className="w-5 h-5" /><span>{t('layout.auth.signIn')}</span>
                </Link>
                <Link to="/register" className="flex items-center space-x-3 px-4 py-3 text-sky-300" onClick={() => setMenuOpen(false)}>
                  <User className="w-5 h-5" /><span>{t('layout.auth.getStarted')}</span>
                </Link>
              </>
            )}
            <div className="px-4 py-3 border-t border-slate-700/50">
              <label className="block text-[11px] font-medium text-slate-400 mb-1">
                {t('layout.lang.label')}
              </label>
              <select
                value={language}
                onChange={(event) => setLanguage(event.target.value as typeof language)}
                className="w-full bg-slate-800 border border-slate-700 text-slate-200 text-sm rounded px-2 py-1.5 focus:outline-none"
              >
                {languages.map((lang) => (
                  <option key={lang.code} value={lang.code}>
                    {lang.label}
                  </option>
                ))}
              </select>
            </div>
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
            {t('layout.footer.builtByPrefix')}{' '}
            <a
              href="https://github.com/FirstPrinciples-Sun"
              className="text-sky-300 hover:text-sky-200"
              target="_blank"
              rel="noopener noreferrer"
            >
              FirstPrinciples-Sun
            </a>
            {' '}{t('layout.footer.builtBySuffix')}
          </p>
          <p className="text-xs mt-2 text-slate-500">
            {t('layout.footer.license')}
          </p>
        </div>
      </footer>
    </div>
  )
}
