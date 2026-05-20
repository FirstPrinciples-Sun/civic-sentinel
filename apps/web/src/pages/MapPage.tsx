import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { MapContainer, TileLayer, Marker, Popup, useMap } from 'react-leaflet'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import { useIssues } from '../hooks/useIssues'
import type { Issue } from '../services/api'
import {
  Loader2,
  Search,
  MapPin,
  Calendar,
  ChevronLeft,
  ChevronRight,
  Info,
} from 'lucide-react'
import { useI18n } from '../context/LanguageContext'

const DEFAULT_CENTER: [number, number] = [13.7563, 100.5018]

const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'reported':
      return '#f59e0b'
    case 'underreview':
      return '#0ea5e9'
    case 'inprogress':
      return '#38bdf8'
    case 'resolved':
      return '#10b981'
    case 'closed':
      return '#64748b'
    case 'escalated':
      return '#ef4444'
    default:
      return '#0ea5e9'
  }
}

const getStatusBadgeClass = (status: string) => {
  switch (status.toLowerCase()) {
    case 'reported':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'underreview':
      return 'text-sky-400 bg-sky-500/10 border-sky-500/20'
    case 'inprogress':
      return 'text-cyan-300 bg-cyan-500/10 border-cyan-500/20'
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

const getPriorityBadgeClass = (priority: string) => {
  switch (priority.toLowerCase()) {
    case 'low':
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
    case 'medium':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'high':
      return 'text-orange-400 bg-orange-500/10 border-orange-500/20'
    case 'critical':
      return 'text-red-400 bg-red-500/10 border-red-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
  }
}

const createCustomIcon = (status: string) => {
  const color = getStatusColor(status)
  return L.divIcon({
    html: `
      <div class="relative w-8 h-8 flex items-center justify-center">
        <div class="absolute w-8 h-8 rounded-full opacity-35 animate-ping" style="background-color: ${color}"></div>
        <svg class="w-8 h-8 filter drop-shadow-md z-10" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2C8.13 2 5 5.13 5 9C5 14.25 12 22 12 22C12 22 19 14.25 19 9C19 5.13 15.87 2 12 2ZM12 11.5C10.62 11.5 9.5 10.38 9.5 9C9.5 7.62 10.62 6.5 12 6.5C13.38 6.5 14.5 7.62 14.5 9C14.5 10.38 13.38 11.5 12 11.5Z" fill="${color}"/>
        </svg>
      </div>
    `,
    className: 'custom-marker-icon',
    iconSize: [32, 32],
    iconAnchor: [16, 32],
    popupAnchor: [0, -32],
  })
}

function MapViewport({
  filteredIssues,
  selectedIssue,
}: {
  filteredIssues: Issue[]
  selectedIssue: Issue | null
}) {
  const map = useMap()

  useEffect(() => {
    if (selectedIssue) {
      map.setView([selectedIssue.location.latitude, selectedIssue.location.longitude], 15, {
        animate: true,
        duration: 1.2,
      })
      return
    }

    if (filteredIssues.length === 0) {
      map.setView(DEFAULT_CENTER, 12, { animate: true })
      return
    }

    const bounds = L.latLngBounds(
      filteredIssues.map((issue) => [issue.location.latitude, issue.location.longitude]),
    )
    map.fitBounds(bounds, {
      padding: [44, 44],
      maxZoom: 13,
      animate: true,
    })
  }, [filteredIssues, map, selectedIssue])

  return null
}

export default function MapPage() {
  const { t, formatCategory, formatPriority, formatStatus, locale } = useI18n()
  const { data: issuesResult, isLoading } = useIssues(1, 100)
  const [selectedIssue, setSelectedIssue] = useState<Issue | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [categoryFilter, setCategoryFilter] = useState('all')
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)

  const issues = issuesResult?.issues ?? []

  const filteredIssues = issues.filter((issue) => {
    const matchesSearch =
      issue.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      issue.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (issue.location.address &&
        issue.location.address.toLowerCase().includes(searchTerm.toLowerCase()))
    const matchesStatus =
      statusFilter === 'all' || issue.status.toLowerCase() === statusFilter.toLowerCase()
    const matchesCategory =
      categoryFilter === 'all' || issue.category.toLowerCase() === categoryFilter.toLowerCase()
    return matchesSearch && matchesStatus && matchesCategory
  })

  const formatDate = (value: string) => {
    return new Intl.DateTimeFormat(locale, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    }).format(new Date(value))
  }

  return (
    <div className="h-[calc(100vh-4rem)] relative overflow-hidden bg-slate-950">
      <MapContainer
        center={DEFAULT_CENTER}
        zoom={12}
        className="h-full w-full z-10"
        scrollWheelZoom={true}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <MapViewport filteredIssues={filteredIssues} selectedIssue={selectedIssue} />

        {filteredIssues.map((issue) => (
          <Marker
            key={issue.id}
            position={[issue.location.latitude, issue.location.longitude]}
            icon={createCustomIcon(issue.status)}
            eventHandlers={{
              click: () => {
                setSelectedIssue(issue)
              },
            }}
          >
            <Popup>
              <div className="p-1 max-w-sm text-slate-800 font-sans">
                <h3 className="font-bold text-base mb-1 leading-tight text-slate-900">
                  {issue.title}
                </h3>
                <div className="flex flex-wrap gap-1.5 mb-2">
                  <span className={`px-2 py-0.5 text-xs font-semibold rounded border ${getStatusBadgeClass(issue.status)}`}>
                    {formatStatus(issue.status)}
                  </span>
                  <span className={`px-2 py-0.5 text-xs font-semibold rounded border ${getPriorityBadgeClass(issue.priority)}`}>
                    {formatPriority(issue.priority)}
                  </span>
                </div>
                <p className="text-sm text-slate-600 mb-3 line-clamp-3">
                  {issue.description}
                </p>
                {issue.location.address && (
                  <div className="flex items-center gap-1 text-xs text-slate-500 border-t pt-2 border-slate-200">
                    <MapPin className="w-3.5 h-3.5 flex-shrink-0 text-slate-400" />
                    <span className="truncate">{issue.location.address}</span>
                  </div>
                )}
                <div className="mt-2 pt-2 border-t border-slate-200 text-[11px] text-slate-500 space-y-0.5">
                  <p>{t('map.createdAt')}: {formatDate(issue.created_at)}</p>
                  <p>{t('map.updatedAt')}: {formatDate(issue.updated_at)}</p>
                </div>
                <Link
                  to={`/issues/${issue.id}`}
                  className="inline-block mt-2 text-xs text-sky-700 hover:text-sky-800 font-semibold"
                >
                  {t('map.viewIssueDetails')}
                </Link>
              </div>
            </Popup>
          </Marker>
        ))}
      </MapContainer>

      <div
        className={`absolute top-4 left-4 z-[1000] max-h-[calc(100vh-6rem)] w-80 glass-panel flex flex-col pointer-events-auto transition-all duration-300 shadow-2xl overflow-hidden border-slate-700/60 bg-slate-900/85 backdrop-blur-xl ${
          isSidebarOpen ? 'translate-x-0' : '-translate-x-[calc(100%+1rem)]'
        }`}
      >
        <button
          onClick={() => setIsSidebarOpen(!isSidebarOpen)}
          className="absolute -right-10 top-4 w-10 h-10 bg-slate-800 border-r border-y border-slate-700/50 rounded-r-lg flex items-center justify-center shadow-lg text-sky-300 hover:text-sky-200 transition-colors pointer-events-auto"
        >
          {isSidebarOpen ? <ChevronLeft className="w-5 h-5" /> : <ChevronRight className="w-5 h-5" />}
        </button>

        <div className="p-4 border-b border-slate-700/50">
          <h2 className="text-lg font-bold gradient-text mb-1">{t('map.title')}</h2>
          <p className="text-xs text-slate-400">{t('map.subtitle')}</p>
        </div>

        <div className="p-4 space-y-3 border-b border-slate-700/50 bg-slate-800/20">
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder={t('map.search')}
              value={searchTerm}
              onChange={(event) => setSearchTerm(event.target.value)}
              className="w-full pl-9 pr-4 py-2 bg-slate-800/80 border border-slate-700/50 rounded-lg text-sm placeholder-slate-500 focus:outline-none focus:border-sky-500/50 transition-colors text-white"
            />
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-[10px] font-semibold text-slate-400 mb-1 uppercase tracking-wider">{t('map.status')}</label>
              <select
                value={statusFilter}
                onChange={(event) => setStatusFilter(event.target.value)}
                className="w-full px-2 py-1.5 bg-slate-800/80 border border-slate-700/50 rounded-lg text-xs focus:outline-none focus:border-sky-500/50 transition-colors text-white"
              >
                <option value="all">{t('map.filter.all')}</option>
                <option value="reported">{formatStatus('reported')}</option>
                <option value="underreview">{t('map.filter.reviewing')}</option>
                <option value="inprogress">{formatStatus('inprogress')}</option>
                <option value="resolved">{formatStatus('resolved')}</option>
                <option value="closed">{formatStatus('closed')}</option>
                <option value="escalated">{formatStatus('escalated')}</option>
              </select>
            </div>

            <div>
              <label className="block text-[10px] font-semibold text-slate-400 mb-1 uppercase tracking-wider">{t('map.category')}</label>
              <select
                value={categoryFilter}
                onChange={(event) => setCategoryFilter(event.target.value)}
                className="w-full px-2 py-1.5 bg-slate-800/80 border border-slate-700/50 rounded-lg text-xs focus:outline-none focus:border-sky-500/50 transition-colors text-white"
              >
                <option value="all">{t('map.filter.all')}</option>
                <option value="infrastructure">{formatCategory('infrastructure')}</option>
                <option value="safety">{formatCategory('safety')}</option>
                <option value="environment">{formatCategory('environment')}</option>
                <option value="sanitation">{formatCategory('sanitation')}</option>
                <option value="transportation">{formatCategory('transportation')}</option>
                <option value="publicutility">{formatCategory('publicutility')}</option>
                <option value="other">{formatCategory('other')}</option>
              </select>
            </div>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto divide-y divide-slate-800/60 custom-scrollbar">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 text-slate-400">
              <Loader2 className="w-8 h-8 animate-spin text-sky-300 mb-2" />
              <span className="text-xs">{t('map.loading')}</span>
            </div>
          ) : filteredIssues.length === 0 ? (
            <div className="text-center py-12 text-slate-500 px-4">
              <Info className="w-6 h-6 mx-auto mb-2 text-slate-600" />
              <p className="text-xs">{t('map.none')}</p>
            </div>
          ) : (
            filteredIssues.map((issue) => (
              <div
                key={issue.id}
                onClick={() => setSelectedIssue(issue)}
                className={`p-3 text-left transition-all duration-150 cursor-pointer ${
                  selectedIssue?.id === issue.id
                    ? 'bg-sky-500/10 border-l-2 border-sky-500'
                    : 'hover:bg-slate-800/35 border-l-2 border-transparent'
                }`}
              >
                <h4 className="font-semibold text-sm text-slate-100 truncate mb-1">
                  {issue.title}
                </h4>
                <p className="text-xs text-slate-400 line-clamp-2 mb-2">
                  {issue.description}
                </p>
                <div className="flex items-center justify-between">
                  <span className={`px-1.5 py-0.5 text-[10px] font-bold rounded border ${getStatusBadgeClass(issue.status)}`}>
                    {formatStatus(issue.status)}
                  </span>
                  <span className="text-[10px] text-slate-500 flex items-center gap-0.5">
                    <Calendar className="w-3 h-3" />
                    {t('map.createdAt')}: {formatDate(issue.created_at)}
                  </span>
                </div>
                <Link
                  to={`/issues/${issue.id}`}
                  onClick={(event) => event.stopPropagation()}
                  className="inline-block mt-2 text-[11px] text-sky-300 hover:text-sky-200"
                >
                  {t('map.viewDetails')}
                </Link>
              </div>
            ))
          )}
        </div>
      </div>

      {!isSidebarOpen && (
        <button
          onClick={() => setIsSidebarOpen(true)}
          className="absolute top-4 left-4 z-[1000] w-10 h-10 bg-slate-800/90 border border-slate-700/50 rounded-lg flex items-center justify-center shadow-lg text-sky-300 hover:text-sky-200 pointer-events-auto"
        >
          <ChevronRight className="w-5 h-5" />
        </button>
      )}
    </div>
  )
}
