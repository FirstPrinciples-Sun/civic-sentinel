import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { LogIn, Mail, Lock, Shield } from 'lucide-react'
import { useAuth } from '../context/AuthContext'
import { useI18n } from '../context/LanguageContext'
import toast from 'react-hot-toast'

export default function LoginPage() {
  const navigate = useNavigate()
  const { login, isLoading } = useAuth()
  const { t } = useI18n()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await login(email, password)
      toast.success(t('login.success'))
      navigate('/')
    } catch (err: any) {
      const msg = err?.response?.data?.error || t('login.failed')
      toast.error(msg)
    }
  }

  return (
    <div className="min-h-[calc(100vh-4rem)] flex items-center justify-center px-4 py-12">
      <div className="w-full max-w-md">
        <div className="glass-panel rounded-2xl p-8 shadow-xl">
          <div className="text-center mb-8">
            <Shield className="w-12 h-12 text-sky-300 mx-auto mb-4" />
            <h1 className="text-2xl font-bold text-white">{t('login.title')}</h1>
            <p className="text-slate-400 mt-2">{t('login.subtitle')}</p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1.5">{t('login.email')}</label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  className="w-full pl-10 pr-4 py-2.5 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-sky-500/50 focus:ring-1 focus:ring-sky-500/30"
                  placeholder={t('report.placeholder.email')}
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1.5">{t('login.password')}</label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500" />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  className="w-full pl-10 pr-4 py-2.5 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-sky-500/50 focus:ring-1 focus:ring-sky-500/30"
                  placeholder="••••••••"
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-2.5 bg-sky-700 hover:bg-sky-600 disabled:opacity-50 text-white rounded-lg font-medium flex items-center justify-center gap-2 transition-colors"
            >
              {isLoading ? t('login.loading') : <><LogIn className="w-4 h-4" /> {t('login.submit')}</>}
            </button>
          </form>

          <p className="text-center text-slate-400 text-sm mt-6">
            {t('login.noAccount')}{' '}
            <Link to="/register" className="text-sky-300 hover:text-sky-200 font-medium">
              {t('login.createOne')}
            </Link>
          </p>
        </div>
      </div>
    </div>
  )
}
