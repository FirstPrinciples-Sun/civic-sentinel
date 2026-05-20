import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { AlertTriangle, CheckCircle2, Filter, Loader2, ShieldAlert, Timer } from 'lucide-react'
import toast from 'react-hot-toast'
import { useIssues, updateIssueStatus } from '../hooks/useIssues'
import { useAuth } from '../context/AuthContext'
import { useI18n } from '../context/LanguageContext'
import type { Issue, IssueFilters } from '../services/api'

type StatusValue = 'reported' | 'underreview' | 'inprogress' | 'resolved' | 'closed' | 'escalated'
type VerificationValue = 'trusted' | 'needs_review' | 'suspicious'

type DraftStatusUpdate = {
  status: StatusValue
  reason: string
}

const STATUS_SEQUENCE: StatusValue[] = [
  'reported',
  'underreview',
  'inprogress',
  'resolved',
  'closed',
  'escalated',
]

const STATUS_ORDER: Record<StatusValue, number> = {
  reported: 1,
  underreview: 2,
  inprogress: 3,
  resolved: 4,
  closed: 5,
  escalated: 6,
}

const ACTIVE_STATUSES: StatusValue[] = ['reported', 'underreview', 'inprogress', 'escalated']

function verificationBadgeClass(value: string): string {
  switch (value) {
    case 'trusted':
      return 'text-emerald-300 bg-emerald-500/10 border-emerald-500/30'
    case 'needs_review':
      return 'text-amber-300 bg-amber-500/10 border-amber-500/30'
    case 'suspicious':
      return 'text-rose-300 bg-rose-500/10 border-rose-500/30'
    default:
      return 'text-slate-300 bg-slate-500/10 border-slate-500/30'
  }
}

function statusBadgeClass(status: string): string {
  switch (status) {
    case 'reported':
      return 'text-amber-300 bg-amber-500/10 border-amber-500/30'
    case 'underreview':
      return 'text-cyan-300 bg-cyan-500/10 border-cyan-500/30'
    case 'inprogress':
      return 'text-sky-300 bg-sky-500/10 border-sky-500/30'
    case 'resolved':
      return 'text-emerald-300 bg-emerald-500/10 border-emerald-500/30'
    case 'closed':
      return 'text-slate-300 bg-slate-500/10 border-slate-500/30'
    case 'escalated':
      return 'text-rose-300 bg-rose-500/10 border-rose-500/30'
    default:
      return 'text-slate-300 bg-slate-500/10 border-slate-500/30'
  }
}

function needsReasonForTransition(current: StatusValue, next: StatusValue): boolean {
  if (current === next) return false
  if (current === 'closed' && next !== 'closed') return true
  return STATUS_ORDER[next] < STATUS_ORDER[current]
}

export default function AdminPage() {
  const { user, isAuthenticated } = useAuth()
  const { t, formatCategory, formatPriority, formatStatus, locale } = useI18n()
  const queryClient = useQueryClient()

  const [statusFilter, setStatusFilter] = useState<'all' | StatusValue>('all')
  const [verificationFilter, setVerificationFilter] = useState<'all' | VerificationValue>('all')
  const [sortBy, setSortBy] = useState<'triage' | 'created_at' | 'updated_at'>('triage')
  const [searchTerm, setSearchTerm] = useState('')
  const [drafts, setDrafts] = useState<Record<string, DraftStatusUpdate>>({})

  const hasAdminAccess = isAuthenticated && (user?.role === 'admin' || user?.role === 'responder')
  const issueFilters = useMemo<IssueFilters>(() => {
    return {
      status: statusFilter === 'all' ? undefined : statusFilter,
      verification_state: verificationFilter === 'all' ? undefined : verificationFilter,
      sort: sortBy,
    }
  }, [statusFilter, verificationFilter, sortBy])

  const { data, isLoading, error } = useIssues(1, 100, issueFilters)
  const issues = data?.issues ?? []

  const filteredIssues = useMemo(() => {
    const query = searchTerm.trim().toLowerCase()
    if (!query) return issues

    return issues.filter((issue) => {
      return (
        issue.title.toLowerCase().includes(query) ||
        issue.description.toLowerCase().includes(query) ||
        issue.location.address?.toLowerCase().includes(query)
      )
    })
  }, [issues, searchTerm])

  const summary = useMemo(() => {
    const open = issues.filter((issue) => ACTIVE_STATUSES.includes(issue.status as StatusValue)).length
    const suspicious = issues.filter((issue) => issue.verification_state === 'suspicious').length
    const needReview = issues.filter((issue) => issue.verification_state === 'needs_review').length
    const highTriage = issues.filter((issue) => issue.triage_score >= 90).length

    return { open, suspicious, needReview, highTriage }
  }, [issues])

  const updateMutation = useMutation({
    mutationFn: async ({
      issueId,
      status,
      reason,
    }: {
      issueId: string
      status: StatusValue
      reason: string
    }) => {
      return updateIssueStatus(issueId, {
        status,
        reason: reason.trim() || undefined,
      })
    },
    onSuccess: () => {
      toast.success(t('admin.updateSuccess'))
      void queryClient.invalidateQueries({ queryKey: ['issues'] })
      void queryClient.invalidateQueries({ queryKey: ['issue'] })
      void queryClient.invalidateQueries({ queryKey: ['issue-history'] })
    },
    onError: (err: unknown) => {
      const errorMessage =
        err && typeof err === 'object' && 'response' in err
          ? (err as { response?: { data?: { error?: string } } }).response?.data?.error
          : undefined
      toast.error(errorMessage ?? t('admin.updateFailed'))
    },
  })

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString(locale, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    })
  }

  const getDraft = (issue: Issue): DraftStatusUpdate => {
    return drafts[issue.id] ?? {
      status: issue.status as StatusValue,
      reason: '',
    }
  }

  const updateDraft = (
    issueId: string,
    initialStatus: StatusValue,
    patch: Partial<DraftStatusUpdate>,
  ) => {
    setDrafts((prev) => {
      const current = prev[issueId] ?? { status: initialStatus, reason: '' }
      return {
        ...prev,
        [issueId]: {
          ...current,
          ...patch,
        },
      }
    })
  }

  const applyUpdate = (issue: Issue) => {
    const draft = getDraft(issue)
    const currentStatus = issue.status as StatusValue
    const nextStatus = draft.status
    const requiresReason = needsReasonForTransition(currentStatus, nextStatus)

    if (nextStatus === currentStatus) {
      toast.error(t('admin.statusUnchanged'))
      return
    }

    if (requiresReason && !draft.reason.trim()) {
      toast.error(t('admin.reasonRequired'))
      return
    }

    updateMutation.mutate({
      issueId: issue.id,
      status: nextStatus,
      reason: draft.reason,
    })
  }

  if (!hasAdminAccess) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-16">
        <div className="glass-panel p-8 text-center">
          <ShieldAlert className="w-10 h-10 text-amber-300 mx-auto mb-3" />
          <h1 className="text-2xl font-bold mb-2">{t('admin.lockedTitle')}</h1>
          <p className="text-slate-300 mb-6">{t('admin.lockedDesc')}</p>
          <div className="flex flex-wrap justify-center gap-3">
            <Link className="btn-secondary" to="/">
              {t('admin.backHome')}
            </Link>
            {!isAuthenticated && (
              <Link className="btn-primary" to="/login">
                {t('layout.auth.signIn')}
              </Link>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-12 space-y-6">
      <div className="flex flex-col md:flex-row md:items-end md:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold gradient-text">{t('admin.title')}</h1>
          <p className="text-slate-300 mt-2">{t('admin.subtitle')}</p>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="glass-panel p-4">
          <p className="text-xs text-slate-400">{t('admin.summary.open')}</p>
          <p className="text-2xl font-bold">{summary.open}</p>
        </div>
        <div className="glass-panel p-4">
          <p className="text-xs text-slate-400">{t('admin.summary.needReview')}</p>
          <p className="text-2xl font-bold text-amber-300">{summary.needReview}</p>
        </div>
        <div className="glass-panel p-4">
          <p className="text-xs text-slate-400">{t('admin.summary.suspicious')}</p>
          <p className="text-2xl font-bold text-rose-300">{summary.suspicious}</p>
        </div>
        <div className="glass-panel p-4">
          <p className="text-xs text-slate-400">{t('admin.summary.highTriage')}</p>
          <p className="text-2xl font-bold text-sky-300">{summary.highTriage}</p>
        </div>
      </div>

      <div className="glass-panel p-4 space-y-3">
        <div className="flex items-center gap-2 text-slate-200">
          <Filter className="w-4 h-4 text-sky-300" />
          <p className="text-sm font-semibold">{t('admin.filters.title')}</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
          <input
            value={searchTerm}
            onChange={(event) => setSearchTerm(event.target.value)}
            placeholder={t('admin.filters.search')}
            className="px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm"
          />
          <select
            value={statusFilter}
            onChange={(event) => setStatusFilter(event.target.value as 'all' | StatusValue)}
            className="px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm"
          >
            <option value="all">{t('admin.filters.status.all')}</option>
            {STATUS_SEQUENCE.map((status) => (
              <option key={status} value={status}>
                {formatStatus(status)}
              </option>
            ))}
          </select>
          <select
            value={verificationFilter}
            onChange={(event) =>
              setVerificationFilter(event.target.value as 'all' | VerificationValue)
            }
            className="px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm"
          >
            <option value="all">{t('admin.filters.verification.all')}</option>
            <option value="trusted">{t('admin.filters.verification.trusted')}</option>
            <option value="needs_review">{t('admin.filters.verification.needsReview')}</option>
            <option value="suspicious">{t('admin.filters.verification.suspicious')}</option>
          </select>
          <select
            value={sortBy}
            onChange={(event) => setSortBy(event.target.value as 'triage' | 'created_at' | 'updated_at')}
            className="px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm"
          >
            <option value="triage">{t('admin.filters.sort.triage')}</option>
            <option value="created_at">{t('admin.filters.sort.created')}</option>
            <option value="updated_at">{t('admin.filters.sort.updated')}</option>
          </select>
        </div>
      </div>

      <div className="space-y-3">
        {isLoading ? (
          <div className="glass-panel p-10 text-center">
            <Loader2 className="w-7 h-7 animate-spin mx-auto text-sky-300 mb-2" />
            <p className="text-sm text-slate-300">{t('admin.loading')}</p>
          </div>
        ) : error ? (
          <div className="glass-panel p-8 text-center text-rose-300">
            <AlertTriangle className="w-6 h-6 mx-auto mb-2" />
            <p>{t('admin.loadFailed')}</p>
          </div>
        ) : filteredIssues.length === 0 ? (
          <div className="glass-panel p-8 text-center text-slate-300">
            <CheckCircle2 className="w-6 h-6 mx-auto mb-2 text-emerald-300" />
            <p>{t('admin.empty')}</p>
          </div>
        ) : (
          filteredIssues.map((issue) => {
            const draft = getDraft(issue)
            const requiresReason = needsReasonForTransition(
              issue.status as StatusValue,
              draft.status,
            )
            const isBusy = updateMutation.isPending

            return (
              <div key={issue.id} className="glass-panel p-4">
                <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
                  <div className="space-y-2 min-w-0">
                    <div className="flex flex-wrap gap-2">
                      <span
                        className={`text-xs px-2 py-1 rounded-full border ${statusBadgeClass(issue.status)}`}
                      >
                        {formatStatus(issue.status)}
                      </span>
                      <span
                        className={`text-xs px-2 py-1 rounded-full border ${verificationBadgeClass(issue.verification_state)}`}
                      >
                        {issue.verification_state}
                      </span>
                      <span className="text-xs px-2 py-1 rounded-full border border-sky-500/30 bg-sky-500/10 text-sky-200">
                        {t('admin.triageScore')}: {issue.triage_score}
                      </span>
                    </div>

                    <h3 className="text-lg font-semibold text-slate-100">{issue.title}</h3>
                    <p className="text-sm text-slate-300 line-clamp-2">{issue.description}</p>

                    <div className="flex flex-wrap gap-4 text-xs text-slate-400">
                      <span>{formatCategory(issue.category)}</span>
                      <span>{formatPriority(issue.priority)}</span>
                      <span>{t('admin.corroboration')}: {issue.corroboration_count}</span>
                      {issue.duplicate_of && (
                        <span className="text-amber-300">
                          {t('admin.duplicateOf')}: {issue.duplicate_of.slice(0, 8)}
                        </span>
                      )}
                      <span className="inline-flex items-center gap-1">
                        <Timer className="w-3.5 h-3.5" />
                        {formatDate(issue.created_at)}
                      </span>
                    </div>
                  </div>

                  <div className="w-full lg:w-[360px] space-y-2">
                    <select
                      value={draft.status}
                      onChange={(event) =>
                        updateDraft(issue.id, issue.status as StatusValue, {
                          status: event.target.value as StatusValue,
                        })
                      }
                      className="w-full px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm"
                    >
                      {STATUS_SEQUENCE.map((status) => (
                        <option key={status} value={status}>
                          {formatStatus(status)}
                        </option>
                      ))}
                    </select>
                    <textarea
                      value={draft.reason}
                      onChange={(event) =>
                        updateDraft(issue.id, issue.status as StatusValue, {
                          reason: event.target.value,
                        })
                      }
                      placeholder={
                        requiresReason
                          ? t('admin.reasonRequiredHint')
                          : t('admin.reasonOptionalHint')
                      }
                      className="w-full px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm resize-none"
                      rows={2}
                    />
                    <div className="flex items-center justify-end gap-2">
                      <Link to={`/issues/${issue.id}`} className="btn-secondary px-3 py-2 text-sm">
                        {t('map.viewDetails')}
                      </Link>
                      <button
                        type="button"
                        onClick={() => applyUpdate(issue)}
                        disabled={isBusy}
                        className="btn-primary px-3 py-2 text-sm disabled:opacity-50"
                      >
                        {isBusy ? t('admin.updating') : t('admin.apply')}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
