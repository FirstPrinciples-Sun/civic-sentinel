import { useEffect, useMemo, useRef, useState } from 'react'
import type { ClipboardEvent, KeyboardEvent } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Send, AlertTriangle, Loader2, Upload, X, MapPin, Navigation, Smartphone } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { MapContainer, Marker, TileLayer, useMap, useMapEvents } from 'react-leaflet'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import toast from 'react-hot-toast'
import { useI18n } from '../context/LanguageContext'
import { api, type ApiResponse, type Issue } from '../services/api'

type ReportCategory =
  | 'infrastructure'
  | 'safety'
  | 'environment'
  | 'sanitation'
  | 'transportation'
  | 'publicutility'
  | 'other'

type ReportForm = {
  title: string
  description: string
  category: ReportCategory
  latitude: number
  longitude: number
  address: string
}

type UploadedMedia = {
  url: string
  fileName: string
  previewUrl: string
}

type GeocodeResult = {
  lat: string
  lon: string
  display_name: string
}

const DEFAULT_COORDS: [number, number] = [13.7563, 100.5018]
const OTP_CODE_LENGTH = 6
const OTP_RESEND_COOLDOWN_SECONDS = 60
const OTP_COUNTRIES = [
  { code: 'TH', dialCode: '+66', label: 'Thailand' },
  { code: 'US', dialCode: '+1', label: 'United States' },
  { code: 'JP', dialCode: '+81', label: 'Japan' },
  { code: 'SG', dialCode: '+65', label: 'Singapore' },
  { code: 'MY', dialCode: '+60', label: 'Malaysia' },
] as const

function toE164Phone(dialCode: string, localNumber: string): string {
  const trimmed = localNumber.trim()
  if (!trimmed) return ''

  const digitsOnly = trimmed.replace(/\D/g, '')
  if (!digitsOnly) return ''

  if (trimmed.startsWith('+')) {
    return `+${digitsOnly}`
  }

  const dialDigits = dialCode.replace('+', '')
  let normalized = digitsOnly
  if (normalized.startsWith('00')) {
    normalized = normalized.slice(2)
  }
  if (normalized.startsWith(dialDigits)) {
    return `+${normalized}`
  }

  normalized = normalized.replace(/^0+/, '')
  return `+${dialDigits}${normalized}`
}

const REPORT_MARKER_ICON = L.divIcon({
  html: `
    <div class="relative w-8 h-8 flex items-center justify-center">
      <div class="absolute w-8 h-8 rounded-full opacity-35 animate-ping" style="background-color:#0284c7"></div>
      <svg class="w-8 h-8 filter drop-shadow-md z-10" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M12 2C8.13 2 5 5.13 5 9C5 14.25 12 22 12 22C12 22 19 14.25 19 9C19 5.13 15.87 2 12 2ZM12 11.5C10.62 11.5 9.5 10.38 9.5 9C9.5 7.62 10.62 6.5 12 6.5C13.38 6.5 14.5 7.62 14.5 9C14.5 10.38 13.38 11.5 12 11.5Z" fill="#0284c7"/>
      </svg>
    </div>
  `,
  className: 'report-location-marker',
  iconSize: [32, 32],
  iconAnchor: [16, 32],
})

function LocationMarker({
  position,
  onChange,
}: {
  position: [number, number]
  onChange: (latitude: number, longitude: number) => void
}) {
  const map = useMap()

  useMapEvents({
    click(event) {
      onChange(event.latlng.lat, event.latlng.lng)
    },
  })

  useEffect(() => {
    map.setView(position, Math.max(map.getZoom(), 14), { animate: true })
  }, [map, position])

  return (
    <Marker
      position={position}
      icon={REPORT_MARKER_ICON}
      draggable
      eventHandlers={{
        dragend(event) {
          const marker = event.target as L.Marker
          const next = marker.getLatLng()
          onChange(next.lat, next.lng)
        },
      }}
    />
  )
}

export default function ReportPage() {
  const { t, formatCategory, locale } = useI18n()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isUploading, setIsUploading] = useState(false)
  const [isLocating, setIsLocating] = useState(false)
  const [isSearchingLocation, setIsSearchingLocation] = useState(false)
  const [uploadedMedia, setUploadedMedia] = useState<UploadedMedia[]>([])
  const [priorityHint, setPriorityHint] = useState<string | null>(null)
  const [locationQuery, setLocationQuery] = useState('')
  const [otpCountryCode, setOtpCountryCode] =
    useState<(typeof OTP_COUNTRIES)[number]['code']>('TH')
  const [otpLocalPhone, setOtpLocalPhone] = useState('')
  const [otpDigits, setOtpDigits] = useState<string[]>(Array(OTP_CODE_LENGTH).fill(''))
  const [otpCooldownSeconds, setOtpCooldownSeconds] = useState(0)
  const [otpVerificationToken, setOtpVerificationToken] = useState<string | null>(null)
  const [otpVerifiedPhone, setOtpVerifiedPhone] = useState<string | null>(null)
  const [isRequestingOtp, setIsRequestingOtp] = useState(false)
  const [isVerifyingOtp, setIsVerifyingOtp] = useState(false)
  const otpInputRefs = useRef<Array<HTMLInputElement | null>>([])
  const navigate = useNavigate()

  const reportSchema = useMemo(
    () =>
      z.object({
        title: z.string().min(5, t('report.validation.titleMin')),
        description: z.string().min(20, t('report.validation.descriptionMin')),
        category: z.enum([
          'infrastructure',
          'safety',
          'environment',
          'sanitation',
          'transportation',
          'publicutility',
          'other',
        ]),
        latitude: z.number().min(-90).max(90),
        longitude: z.number().min(-180).max(180),
        address: z.string(),
      }),
    [t],
  )

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors },
  } = useForm<ReportForm>({
    resolver: zodResolver(reportSchema),
    defaultValues: {
      category: 'infrastructure',
      latitude: DEFAULT_COORDS[0],
      longitude: DEFAULT_COORDS[1],
      address: '',
    },
  })

  const title = watch('title')
  const description = watch('description')
  const latitude = watch('latitude')
  const longitude = watch('longitude')

  const safeLatitude = Number.isFinite(latitude) ? latitude : DEFAULT_COORDS[0]
  const safeLongitude = Number.isFinite(longitude) ? longitude : DEFAULT_COORDS[1]
  const markerPosition = useMemo<[number, number]>(
    () => [safeLatitude, safeLongitude],
    [safeLatitude, safeLongitude],
  )
  const selectedOtpCountry =
    OTP_COUNTRIES.find((country) => country.code === otpCountryCode) ?? OTP_COUNTRIES[0]
  const otpPhone = useMemo(
    () => toE164Phone(selectedOtpCountry.dialCode, otpLocalPhone),
    [otpLocalPhone, selectedOtpCountry.dialCode],
  )
  const otpCode = useMemo(() => otpDigits.join(''), [otpDigits])
  const isOtpCodeComplete = useMemo(
    () => otpDigits.every((digit) => digit.length === 1),
    [otpDigits],
  )

  const updateCoordinates = (nextLat: number, nextLng: number) => {
    if (!Number.isFinite(nextLat) || !Number.isFinite(nextLng)) return
    setValue('latitude', Number(nextLat.toFixed(6)), {
      shouldDirty: true,
      shouldValidate: true,
    })
    setValue('longitude', Number(nextLng.toFixed(6)), {
      shouldDirty: true,
      shouldValidate: true,
    })
    void reverseGeocode(nextLat, nextLng)
  }

  const reverseGeocode = async (nextLat: number, nextLng: number) => {
    if (!Number.isFinite(nextLat) || !Number.isFinite(nextLng)) return
    try {
      const response = await fetch(
        `https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat=${encodeURIComponent(String(nextLat))}&lon=${encodeURIComponent(String(nextLng))}`,
        {
          headers: {
            'Accept-Language': locale,
          },
        },
      )

      if (!response.ok) {
        throw new Error('reverse-geocode-failed')
      }

      const data = (await response.json()) as { display_name?: string }
      setValue('address', data.display_name?.trim() ?? '', { shouldDirty: true })
    } catch {
      setValue('address', '', { shouldDirty: true })
    }
  }

  const searchLocation = async () => {
    const query = locationQuery.trim()
    if (!query) {
      toast.error(t('report.location.geocode.empty'))
      return
    }

    setIsSearchingLocation(true)
    try {
      const response = await fetch(
        `https://nominatim.openstreetmap.org/search?format=jsonv2&limit=1&q=${encodeURIComponent(query)}`,
        {
          headers: {
            'Accept-Language': locale,
          },
        },
      )

      if (!response.ok) {
        throw new Error('search-geocode-failed')
      }

      const results = (await response.json()) as GeocodeResult[]
      const first = results[0]

      if (!first) {
        toast.error(t('report.location.geocode.none'))
        return
      }

      const nextLat = Number(first.lat)
      const nextLng = Number(first.lon)
      setValue('latitude', Number(nextLat.toFixed(6)), { shouldDirty: true, shouldValidate: true })
      setValue('longitude', Number(nextLng.toFixed(6)), { shouldDirty: true, shouldValidate: true })
      setValue('address', first.display_name ?? '', { shouldDirty: true })
    } catch {
      toast.error(t('report.location.geocode.failed'))
    } finally {
      setIsSearchingLocation(false)
    }
  }

  const useCurrentLocation = async () => {
    if (!navigator.geolocation) {
      toast.error(t('report.location.current.failed'))
      return
    }

    setIsLocating(true)
    navigator.geolocation.getCurrentPosition(
      async (position) => {
        const nextLat = position.coords.latitude
        const nextLng = position.coords.longitude
        setValue('latitude', Number(nextLat.toFixed(6)), { shouldDirty: true, shouldValidate: true })
        setValue('longitude', Number(nextLng.toFixed(6)), { shouldDirty: true, shouldValidate: true })
        await reverseGeocode(nextLat, nextLng)
        toast.success(t('report.location.current.success'))
        setIsLocating(false)
      },
      () => {
        toast.error(t('report.location.current.failed'))
        setIsLocating(false)
      },
      {
        enableHighAccuracy: true,
        timeout: 10000,
      },
    )
  }

  useEffect(() => {
    if (otpCooldownSeconds <= 0) return
    const timer = window.setTimeout(() => {
      setOtpCooldownSeconds((prev) => Math.max(prev - 1, 0))
    }, 1000)

    return () => window.clearTimeout(timer)
  }, [otpCooldownSeconds])

  const handleOtpDigitChange = (index: number, value: string) => {
    const numeric = value.replace(/\D/g, '').slice(-1)

    setOtpDigits((prev) => {
      const next = [...prev]
      next[index] = numeric
      return next
    })
    setOtpVerificationToken(null)
    setOtpVerifiedPhone(null)

    if (numeric && index < OTP_CODE_LENGTH - 1) {
      otpInputRefs.current[index + 1]?.focus()
    }
  }

  const handleOtpDigitKeyDown = (index: number, event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Backspace' && !otpDigits[index] && index > 0) {
      otpInputRefs.current[index - 1]?.focus()
    }
  }

  const handleOtpPaste = (event: ClipboardEvent<HTMLDivElement>) => {
    const clipboard = event.clipboardData.getData('text')
    const digits = clipboard.replace(/\D/g, '').slice(0, OTP_CODE_LENGTH)
    if (!digits) return

    event.preventDefault()
    const nextDigits = Array(OTP_CODE_LENGTH)
      .fill('')
      .map((_, index) => digits[index] ?? '')
    setOtpDigits(nextDigits)
    setOtpVerificationToken(null)
    setOtpVerifiedPhone(null)
    const nextFocus = Math.min(digits.length, OTP_CODE_LENGTH - 1)
    otpInputRefs.current[nextFocus]?.focus()
  }

  const requestOtp = async () => {
    if (!otpPhone || otpPhone.replace(/\D/g, '').length < 8) {
      toast.error(t('report.otp.invalidPhone'))
      return
    }

    setIsRequestingOtp(true)
    try {
      const response = await api.post<
        ApiResponse<{ phone: string; expires_in_seconds: number; otp_code?: string | null }>
      >('/auth/otp/request', { phone: otpPhone })

      if (response.data.success) {
        toast.success(t('report.otp.requestSuccess'))
        setOtpCooldownSeconds(OTP_RESEND_COOLDOWN_SECONDS)
        const devOtp = response.data.data?.otp_code
        if (devOtp && /^\d{6}$/.test(devOtp)) {
          setOtpDigits(devOtp.split(''))
        } else {
          setOtpDigits(Array(OTP_CODE_LENGTH).fill(''))
        }
        setOtpVerificationToken(null)
        setOtpVerifiedPhone(null)
        otpInputRefs.current[0]?.focus()
      } else {
        toast.error(response.data.error ?? t('report.otp.failed'))
      }
    } catch (error) {
      if (error && typeof error === 'object' && 'response' in error) {
        const axiosError = error as { response?: { data?: { error?: string } } }
        toast.error(axiosError.response?.data?.error ?? t('report.otp.failed'))
      } else {
        toast.error(t('report.otp.failed'))
      }
    } finally {
      setIsRequestingOtp(false)
    }
  }

  const verifyOtp = async () => {
    if (!otpPhone || !isOtpCodeComplete) {
      toast.error(t('report.otp.failed'))
      return
    }

    setIsVerifyingOtp(true)
    try {
      const response = await api.post<
        ApiResponse<{ phone: string; verification_token: string }>
      >('/auth/otp/verify', {
        phone: otpPhone,
        code: otpCode,
      })

      if (response.data.success) {
        const verification = response.data.data
        setOtpVerificationToken(verification.verification_token)
        setOtpVerifiedPhone(verification.phone)
        toast.success(t('report.otp.verifySuccess'))
      } else {
        toast.error(response.data.error ?? t('report.otp.failed'))
      }
    } catch (error) {
      if (error && typeof error === 'object' && 'response' in error) {
        const axiosError = error as { response?: { data?: { error?: string } } }
        toast.error(axiosError.response?.data?.error ?? t('report.otp.failed'))
      } else {
        toast.error(t('report.otp.failed'))
      }
    } finally {
      setIsVerifyingOtp(false)
    }
  }

  const analyzeIssue = () => {
    const text = `${title} ${description}`.toLowerCase()
    if (text.includes('flood') || text.includes('fire') || text.includes('dangerous')) {
      setPriorityHint(t('report.hint.critical'))
    } else if (text.includes('broken') || text.includes('leaking')) {
      setPriorityHint(t('report.hint.high'))
    } else {
      setPriorityHint(t('report.hint.standard'))
    }
  }

  const onSubmit = async (data: ReportForm) => {
    if (uploadedMedia.length === 0) {
      toast.error(t('report.validation.photoRequired'))
      return
    }

    setIsSubmitting(true)
    try {
      const response = await api.post<ApiResponse<Issue>>('/issues', {
        title: data.title,
        description: data.description,
        category: data.category,
        location: {
          latitude: data.latitude,
          longitude: data.longitude,
          address: data.address?.trim() || undefined,
        },
        media_urls: uploadedMedia.map((media) => media.url),
        tags: [],
        otp_phone: otpVerifiedPhone ?? undefined,
        otp_token: otpVerificationToken ?? undefined,
      })

      if (response.data.success) {
        toast.success(response.data.message ?? t('report.success'))
        const issueId = response.data.data.id
        reset({
          category: 'infrastructure',
          latitude: DEFAULT_COORDS[0],
          longitude: DEFAULT_COORDS[1],
          address: '',
          title: '',
          description: '',
        })
        uploadedMedia.forEach((media) => URL.revokeObjectURL(media.previewUrl))
        setUploadedMedia([])
        setPriorityHint(null)
        setLocationQuery('')
        setOtpCountryCode('TH')
        setOtpLocalPhone('')
        setOtpDigits(Array(OTP_CODE_LENGTH).fill(''))
        setOtpCooldownSeconds(0)
        setOtpVerificationToken(null)
        setOtpVerifiedPhone(null)
        navigate(`/issues/${issueId}`)
      } else {
        toast.error(response.data.error ?? t('report.failed'))
      }
    } catch (error) {
      if (error && typeof error === 'object' && 'response' in error) {
        const axiosError = error as { response?: { data?: { error?: string } } }
        toast.error(axiosError.response?.data?.error ?? t('report.failed'))
      } else {
        toast.error(t('report.failed'))
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  const uploadFiles = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    setIsUploading(true)

    const nextUploaded: UploadedMedia[] = []
    try {
      for (const file of Array.from(files)) {
        const formData = new FormData()
        formData.append('file', file)

        const response = await api.post<
          ApiResponse<{ url: string; file_name: string }>
        >('/uploads', formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
        })

        nextUploaded.push({
          url: response.data.data.url,
          fileName: response.data.data.file_name,
          previewUrl: URL.createObjectURL(file),
        })
      }

      setUploadedMedia((prev) => [...prev, ...nextUploaded])
      toast.success(t('report.upload.success', { count: nextUploaded.length }))
    } catch {
      nextUploaded.forEach((media) => URL.revokeObjectURL(media.previewUrl))
      toast.error(t('report.upload.failed'))
    } finally {
      setIsUploading(false)
    }
  }

  const removeMedia = (index: number) => {
    setUploadedMedia((prev) => {
      const clone = [...prev]
      const removed = clone.splice(index, 1)
      if (removed[0]) {
        URL.revokeObjectURL(removed[0].previewUrl)
      }
      return clone
    })
  }

  return (
    <div className="max-w-4xl mx-auto px-4 py-12">
      <div className="text-center mb-10">
        <h1 className="text-3xl font-bold gradient-text mb-4">{t('report.title')}</h1>
        <p className="text-slate-400">{t('report.subtitle')}</p>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        <div>
          <label className="block text-sm font-medium mb-2">{t('report.field.title')}</label>
          <input
            {...register('title')}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-sky-500 focus:border-transparent outline-none transition-all"
            placeholder={t('report.placeholder.title')}
          />
          {errors.title && <p className="text-red-400 text-sm mt-1">{errors.title.message}</p>}
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t('report.field.category')}</label>
          <select
            {...register('category')}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-sky-500 outline-none"
          >
            <option value="infrastructure">{formatCategory('infrastructure')}</option>
            <option value="safety">{formatCategory('safety')}</option>
            <option value="environment">{formatCategory('environment')}</option>
            <option value="sanitation">{formatCategory('sanitation')}</option>
            <option value="transportation">{formatCategory('transportation')}</option>
            <option value="publicutility">{formatCategory('publicutility')}</option>
            <option value="other">{formatCategory('other')}</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t('report.field.description')}</label>
          <textarea
            {...register('description')}
            rows={4}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-sky-500 outline-none resize-none"
            placeholder={t('report.placeholder.description')}
            onBlur={analyzeIssue}
          />
          {errors.description && <p className="text-red-400 text-sm mt-1">{errors.description.message}</p>}
        </div>

        <div className="glass-panel p-4 space-y-4">
          <div className="flex items-start justify-between gap-4">
            <div>
              <h2 className="text-base font-semibold text-slate-100 flex items-center gap-2">
                <MapPin className="w-4 h-4 text-sky-300" />
                {t('report.location.title')}
              </h2>
              <p className="text-xs text-slate-400 mt-1">{t('report.location.subtitle')}</p>
            </div>
            <button
              type="button"
              onClick={useCurrentLocation}
              disabled={isLocating}
              className="btn-secondary px-3 py-2 text-xs inline-flex items-center gap-2 disabled:opacity-50"
            >
              {isLocating ? <Loader2 className="w-4 h-4 animate-spin" /> : <Navigation className="w-4 h-4" />}
              <span>{t('report.location.current')}</span>
            </button>
          </div>

          <div className="flex flex-col sm:flex-row gap-2">
            <input
              type="text"
              value={locationQuery}
              onChange={(event) => setLocationQuery(event.target.value)}
              placeholder={t('report.location.search')}
              className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-sky-500 outline-none text-sm"
            />
            <button
              type="button"
              onClick={searchLocation}
              disabled={isSearchingLocation}
              className="btn-secondary px-4 py-2 text-sm disabled:opacity-50"
            >
              {isSearchingLocation ? t('report.upload.loading') : t('report.location.searchAction')}
            </button>
          </div>

          <div className="h-72 overflow-hidden rounded-lg border border-slate-700/60">
            <MapContainer center={markerPosition} zoom={14} className="h-full w-full" scrollWheelZoom>
              <TileLayer
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
              />
              <LocationMarker position={markerPosition} onChange={updateCoordinates} />
            </MapContainer>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium mb-2">{t('report.field.latitude')}</label>
              <input
                {...register('latitude', { valueAsNumber: true })}
                className="w-full px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-lg"
                onBlur={() => void reverseGeocode(safeLatitude, safeLongitude)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t('report.field.longitude')}</label>
              <input
                {...register('longitude', { valueAsNumber: true })}
                className="w-full px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-lg"
                onBlur={() => void reverseGeocode(safeLatitude, safeLongitude)}
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">{t('report.field.address')}</label>
            <input
              {...register('address')}
              className="w-full px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-lg"
              placeholder={t('report.placeholder.address')}
            />
          </div>
        </div>

        <div className="glass-panel p-4 space-y-4">
          <div className="flex flex-col gap-1">
            <h2 className="text-base font-semibold text-slate-100 flex items-center gap-2">
              <Smartphone className="w-4 h-4 text-sky-300" />
              {t('report.otp.title')}
            </h2>
            <p className="text-xs text-slate-400">{t('report.otp.subtitle')}</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-[180px_1fr] gap-3">
            <div>
              <label className="block text-sm font-medium mb-2">{t('report.otp.country')}</label>
              <select
                value={otpCountryCode}
                onChange={(event) => {
                  const nextCountry = event.target.value as (typeof OTP_COUNTRIES)[number]['code']
                  setOtpCountryCode(nextCountry)
                  setOtpCooldownSeconds(0)
                  setOtpDigits(Array(OTP_CODE_LENGTH).fill(''))
                  setOtpVerificationToken(null)
                  setOtpVerifiedPhone(null)
                }}
                className="w-full px-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg"
              >
                {OTP_COUNTRIES.map((country) => (
                  <option key={country.code} value={country.code}>
                    {country.label} ({country.dialCode})
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t('report.otp.phone')}</label>
              <div className="flex">
                <span className="inline-flex items-center px-3 rounded-l-lg border border-r-0 border-slate-700 bg-slate-800 text-slate-300 text-sm">
                  {selectedOtpCountry.dialCode}
                </span>
                <input
                  type="tel"
                  value={otpLocalPhone}
                  onChange={(event) => {
                    const next = event.target.value.replace(/[^\d+\-\s()]/g, '')
                    setOtpLocalPhone(next)
                    setOtpCooldownSeconds(0)
                    setOtpDigits(Array(OTP_CODE_LENGTH).fill(''))
                    setOtpVerificationToken(null)
                    setOtpVerifiedPhone(null)
                  }}
                  placeholder={t('report.otp.phonePlaceholder')}
                  className="w-full px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-r-lg"
                />
              </div>
              <p className="text-xs text-slate-500 mt-1">{t('report.otp.normalized')}: {otpPhone || '-'}</p>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">{t('report.otp.code')}</label>
            <div className="flex gap-2" onPaste={handleOtpPaste}>
              {otpDigits.map((digit, index) => (
                <input
                  key={`otp-${index}`}
                  ref={(element) => {
                    otpInputRefs.current[index] = element
                  }}
                  inputMode="numeric"
                  autoComplete={index === 0 ? 'one-time-code' : 'off'}
                  maxLength={1}
                  value={digit}
                  onChange={(event) => handleOtpDigitChange(index, event.target.value)}
                  onKeyDown={(event) => handleOtpDigitKeyDown(index, event)}
                  className="w-11 h-11 text-center text-lg font-semibold bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-sky-500 outline-none"
                />
              ))}
            </div>
            <p className="text-xs text-slate-500 mt-1">{t('report.otp.codeHint')}</p>
          </div>

          <div className="flex flex-wrap gap-2 items-center">
            <button
              type="button"
              onClick={requestOtp}
              disabled={isRequestingOtp || isSubmitting || otpCooldownSeconds > 0}
              className="btn-secondary px-4 py-2 text-sm disabled:opacity-50"
            >
              {isRequestingOtp
                ? t('report.otp.requesting')
                : otpCooldownSeconds > 0
                  ? t('report.otp.resendIn', { seconds: otpCooldownSeconds })
                  : t('report.otp.request')}
            </button>
            <button
              type="button"
              onClick={verifyOtp}
              disabled={isVerifyingOtp || isSubmitting || !isOtpCodeComplete}
              className="btn-secondary px-4 py-2 text-sm disabled:opacity-50"
            >
              {isVerifyingOtp ? t('report.otp.verifying') : t('report.otp.verify')}
            </button>
            {otpVerificationToken && otpVerifiedPhone && (
              <span className="inline-flex items-center px-3 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-sm">
                {t('report.otp.verified')}: {otpVerifiedPhone}
              </span>
            )}
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">{t('report.field.photos')}</label>
          <label className="w-full px-4 py-4 border border-dashed border-slate-600 rounded-lg bg-slate-800/40 hover:bg-slate-800/70 cursor-pointer transition-all flex items-center justify-center gap-2 text-slate-300">
            {isUploading ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin text-sky-300" />
                <span>{t('report.upload.loading')}</span>
              </>
            ) : (
              <>
                <Upload className="w-4 h-4 text-sky-300" />
                <span>{t('report.upload.action')}</span>
              </>
            )}
            <input
              type="file"
              accept="image/*"
              multiple
              className="hidden"
              onChange={(event) => uploadFiles(event.target.files)}
              disabled={isUploading || isSubmitting}
            />
          </label>

          {uploadedMedia.length > 0 && (
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3 mt-3">
              {uploadedMedia.map((media, index) => (
                <div
                  key={`${media.url}-${index}`}
                  className="relative rounded-lg overflow-hidden border border-slate-700"
                >
                  <img
                    src={media.previewUrl}
                    alt={media.fileName}
                    className="w-full h-28 object-cover"
                  />
                  <button
                    type="button"
                    onClick={() => removeMedia(index)}
                    className="absolute top-1 right-1 p-1 rounded-full bg-black/60 text-white hover:bg-black/80"
                    aria-label={t('report.upload.remove')}
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {priorityHint && (
          <div className="glass-panel p-4 border-l-4 border-sky-500">
            <div className="flex items-start space-x-3">
              <AlertTriangle className="w-5 h-5 text-sky-300 mt-0.5" />
              <p className="text-sm text-slate-300">{priorityHint}</p>
            </div>
          </div>
        )}

        <button
          type="submit"
          disabled={
            isSubmitting ||
            isUploading ||
            isLocating ||
            isSearchingLocation ||
            isRequestingOtp ||
            isVerifyingOtp
          }
          className="w-full btn-primary flex items-center justify-center space-x-2 disabled:opacity-50"
        >
          {isSubmitting ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>{t('report.submitting')}</span>
            </>
          ) : (
            <>
              <Send className="w-4 h-4" />
              <span>{t('report.submit')}</span>
            </>
          )}
        </button>
      </form>
    </div>
  )
}
