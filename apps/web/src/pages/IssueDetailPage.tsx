import { FormEvent, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import {
  AlertTriangle,
  ArrowLeft,
  Calendar,
  Clock3,
  Loader2,
  MapPin,
  MessageSquare,
  Send,
  User,
} from 'lucide-react'
import toast from 'react-hot-toast'
import { useAuth } from '../context/AuthContext'
import { useI18n } from '../context/LanguageContext'
import { useIssue, useIssueComments, useIssueHistory } from '../hooks/useIssues'
import { api, type ApiResponse, type CreateCommentRequest, type IssueComment } from '../services/api'

function statusBadgeClass(status: string): string {
  switch (status.toLowerCase()) {
    case 'reported':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'underreview':
      return 'text-blue-400 bg-blue-500/10 border-blue-500/20'
    case 'inprogress':
      return 'text-purple-400 bg-purple-500/10 border-purple-500/20'
    case 'resolved':
      return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20'
    case 'closed':
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
    case 'escalated':
      return 'text-red-400 bg-red-500/10 border-red-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
  }
}

function priorityBadgeClass(priority: string): string {
  switch (priority.toLowerCase()) {
    case 'critical':
      return 'text-red-400 bg-red-500/10 border-red-500/20'
    case 'high':
      return 'text-orange-400 bg-orange-500/10 border-orange-500/20'
    case 'medium':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
  }
}

export default function IssueDetailPage() {
  const { t, formatCategory, formatPriority, formatStatus, locale } = useI18n()
  const { id } = useParams<{ id: string }>()
  const { isAuthenticated, user } = useAuth()
  const queryClient = useQueryClient()

  const { data: issue, isLoading: issueLoading, error: issueError } = useIssue(id)
  const { data: comments, isLoading: commentsLoading } = useIssueComments(id)
  const { data: history, isLoading: historyLoading } = useIssueHistory(id)

  const [commentContent, setCommentContent] = useState('')
  const [isInternal, setIsInternal] = useState(false)
  const canMarkInternal = user?.role === 'admin' || user?.role === 'responder'
  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString(locale, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  const addComment = useMutation({
    mutationFn: async ({ issueId, payload }: { issueId: string; payload: CreateCommentRequest }) => {
      const response = await api.post<ApiResponse<IssueComment>>(
        `/issues/${issueId}/comments`,
        payload,
      )
      return response.data.data
    },
    onSuccess: async () => {
      setCommentContent('')
      setIsInternal(false)
      await queryClient.invalidateQueries({ queryKey: ['issue-comments', id] })
      toast.success(t('issue.comments.success'))
    },
    onError: () => {
      toast.error(t('issue.comments.failed'))
    },
  })

  const handleSubmitComment = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const content = commentContent.trim()
    if (!id || !content) return

    addComment.mutate({
      issueId: id,
      payload: {
        content,
        is_internal: canMarkInternal ? isInternal : false,
      },
    })
  }

  if (issueLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 py-12 flex justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-emerald-400" />
      </div>
    )
  }

  if (issueError || !issue) {
    return (
      <div className="max-w-3xl mx-auto px-4 py-12">
        <div className="glass-panel p-8 text-center">
          <AlertTriangle className="w-8 h-8 mx-auto mb-3 text-red-400" />
          <h1 className="text-2xl font-bold mb-2">{t('issue.notFound.title')}</h1>
          <p className="text-slate-400 mb-6">
            {t('issue.notFound.desc')}
          </p>
          <Link to="/dashboard" className="btn-secondary inline-flex items-center gap-2">
            <ArrowLeft className="w-4 h-4" />
            {t('issue.backDashboard')}
          </Link>
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-7xl mx-auto px-4 py-10">
      <div className="flex flex-wrap items-center gap-3 mb-6">
        <Link to="/dashboard" className="btn-secondary inline-flex items-center gap-2">
          <ArrowLeft className="w-4 h-4" />
          {t('issue.nav.dashboard')}
        </Link>
        <Link to="/map" className="btn-secondary inline-flex items-center gap-2">
          <MapPin className="w-4 h-4" />
          {t('issue.nav.map')}
        </Link>
      </div>

      <section className="glass-panel p-6 mb-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-2xl font-bold text-white break-words">{issue.title}</h1>
            <div className="flex flex-wrap gap-2 mt-3">
              <span className={`px-2 py-1 text-xs rounded border ${statusBadgeClass(issue.status)}`}>
                {formatStatus(issue.status)}
              </span>
              <span className={`px-2 py-1 text-xs rounded border ${priorityBadgeClass(issue.priority)}`}>
                {formatPriority(issue.priority)}
              </span>
              <span className="px-2 py-1 text-xs rounded border text-slate-300 bg-slate-800 border-slate-700">
                {formatCategory(issue.category)}
              </span>
            </div>
          </div>
          <div className="text-xs text-slate-400 space-y-1">
            <p className="flex items-center gap-1">
              <Calendar className="w-3.5 h-3.5" />
              {t('issue.meta.created')}: {formatDate(issue.created_at)}
            </p>
            <p className="flex items-center gap-1">
              <Clock3 className="w-3.5 h-3.5" />
              {t('issue.meta.updated')}: {formatDate(issue.updated_at)}
            </p>
          </div>
        </div>

        <p className="text-slate-300 mt-5 leading-relaxed">{issue.description}</p>

        <div className="mt-5 grid md:grid-cols-2 gap-3 text-sm">
          <div className="bg-slate-900/50 border border-slate-700/60 rounded-lg p-3">
            <p className="text-slate-400">{t('issue.meta.location')}</p>
            <p className="text-slate-200 mt-1">
              {issue.location.address ?? t('issue.meta.noAddress')}
            </p>
            <p className="text-xs text-slate-500 mt-1">
              {issue.location.latitude.toFixed(5)}, {issue.location.longitude.toFixed(5)}
            </p>
          </div>
          <div className="bg-slate-900/50 border border-slate-700/60 rounded-lg p-3">
            <p className="text-slate-400">{t('issue.meta.reporter')}</p>
            <p className="text-slate-200 mt-1 break-all">{issue.reporter_id ?? t('issue.meta.anonymous')}</p>
            <p className="text-xs text-slate-500 mt-1 break-all">
              {t('issue.meta.assignedTo')}: {issue.assigned_to ?? t('issue.meta.unassigned')}
            </p>
          </div>
        </div>
      </section>

      <div className="grid lg:grid-cols-2 gap-6">
        <section className="glass-panel p-6">
          <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-emerald-400" />
            {t('issue.comments.title')}
          </h2>

          {isAuthenticated && user ? (
            <form onSubmit={handleSubmitComment} className="mb-5 space-y-3">
              <textarea
                value={commentContent}
                onChange={(event) => setCommentContent(event.target.value)}
                rows={3}
                placeholder={t('issue.comments.placeholder')}
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 resize-none"
              />
              <div className="flex items-center justify-between gap-3">
                {canMarkInternal ? (
                  <label className="text-xs text-slate-400 flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={isInternal}
                      onChange={(event) => setIsInternal(event.target.checked)}
                      className="accent-emerald-500"
                    />
                    {t('issue.comments.internal')}
                  </label>
                ) : (
                  <span className="text-xs text-slate-500">{t('issue.comments.publicNote')}</span>
                )}
                <button
                  type="submit"
                  disabled={addComment.isPending || !commentContent.trim()}
                  className="btn-primary inline-flex items-center gap-2 disabled:opacity-50"
                >
                  {addComment.isPending ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <Send className="w-4 h-4" />
                  )}
                  <span>{t('issue.comments.add')}</span>
                </button>
              </div>
            </form>
          ) : (
            <div className="mb-5 text-sm text-slate-400 bg-slate-900/60 border border-slate-700/60 rounded-lg p-3">
              {t('issue.comments.noSignIn')}
            </div>
          )}

          {commentsLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-emerald-400" />
            </div>
          ) : !comments || comments.length === 0 ? (
            <p className="text-slate-500 text-sm">{t('issue.comments.none')}</p>
          ) : (
            <div className="space-y-3">
              {comments.map((comment) => (
                <article
                  key={comment.id}
                  className="bg-slate-900/50 border border-slate-700/60 rounded-lg p-3"
                >
                  <div className="flex items-center justify-between gap-2 mb-2">
                    <div className="text-xs text-slate-400 flex items-center gap-1 break-all">
                      <User className="w-3.5 h-3.5" />
                      <span>{comment.author_id}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      {comment.is_internal && (
                        <span className="text-[10px] px-1.5 py-0.5 rounded border border-amber-500/30 bg-amber-500/10 text-amber-400">
                          {t('issue.comments.internalTag')}
                        </span>
                      )}
                      <span className="text-xs text-slate-500">{formatDate(comment.created_at)}</span>
                    </div>
                  </div>
                  <p className="text-sm text-slate-200 whitespace-pre-wrap">{comment.content}</p>
                </article>
              ))}
            </div>
          )}
        </section>

        <section className="glass-panel p-6">
          <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Clock3 className="w-5 h-5 text-emerald-400" />
            {t('issue.history.title')}
          </h2>

          {historyLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-emerald-400" />
            </div>
          ) : !history || history.length === 0 ? (
            <p className="text-slate-500 text-sm">
              {t('issue.history.none')}
            </p>
          ) : (
            <div className="space-y-3">
              {history.map((entry) => (
                <article
                  key={entry.id}
                  className="bg-slate-900/50 border border-slate-700/60 rounded-lg p-3"
                >
                  <div className="flex items-center justify-between gap-2 mb-1">
                    <p className="text-sm text-slate-100">
                      <span className="text-slate-400">{formatStatus(entry.old_status)}</span>
                      {' '}→{' '}
                      <span className="text-emerald-400">{formatStatus(entry.new_status)}</span>
                    </p>
                    <span className="text-xs text-slate-500">{formatDate(entry.created_at)}</span>
                  </div>
                  <p className="text-xs text-slate-400 break-all">{t('issue.history.changedBy')}: {entry.changed_by}</p>
                  {entry.reason && (
                    <p className="text-xs text-slate-300 mt-2">{t('issue.history.reason')}: {entry.reason}</p>
                  )}
                </article>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  )
}
