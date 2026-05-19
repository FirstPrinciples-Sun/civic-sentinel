import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { MapPin, Camera, Send, AlertTriangle } from 'lucide-react'
import toast from 'react-hot-toast'

const reportSchema = z.object({
  title: z.string().min(5, 'Title must be at least 5 characters'),
  description: z.string().min(20, 'Please provide more details'),
  category: z.enum(['infrastructure', 'safety', 'environment', 'sanitation', 'transportation', 'public_utility', 'other']),
  latitude: z.number().min(-90).max(90),
  longitude: z.number().min(-180).max(180),
})

type ReportForm = z.infer<typeof reportSchema>

export default function ReportPage() {
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [aiAnalysis, setAiAnalysis] = useState<string | null>(null)

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<ReportForm>({
    resolver: zodResolver(reportSchema),
    defaultValues: {
      category: 'infrastructure',
      latitude: 13.7563, // Bangkok default
      longitude: 100.5018,
    },
  })

  const title = watch('title')
  const description = watch('description')

  // Simulate AI analysis
  const analyzeIssue = () => {
    const text = `${title} ${description}`.toLowerCase()
    if (text.includes('flood') || text.includes('fire') || text.includes('dangerous')) {
      setAiAnalysis('🚨 AI detected: This issue may be CRITICAL priority. Emergency services will be notified immediately.')
    } else if (text.includes('broken') || text.includes('leaking')) {
      setAiAnalysis('⚠️ AI detected: HIGH priority — infrastructure issue requiring prompt attention.')
    } else {
      setAiAnalysis('✅ AI analysis: Standard priority. Will be reviewed and routed to appropriate department.')
    }
  }

  const onSubmit = async (data: ReportForm) => {
    setIsSubmitting(true)
    try {
      // TODO: Connect to actual API
      await new Promise((resolve) => setTimeout(resolve, 1500))
      toast.success('Issue reported successfully! Our AI is analyzing priority.')
      setAiAnalysis(null)
    } catch {
      toast.error('Failed to submit. Please try again.')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-12">
      <div className="text-center mb-10">
        <h1 className="text-3xl font-bold gradient-text mb-4">Report a Community Issue</h1>
        <p className="text-slate-400">
          Your report helps improve the community. Our AI will analyze and route it to the right responders.
        </p>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        {/* Title */}
        <div>
          <label className="block text-sm font-medium mb-2">Issue Title *</label>
          <input
            {...register('title')}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-transparent outline-none transition-all"
            placeholder="e.g., Broken street light on Main Road"
          />
          {errors.title && <p className="text-red-400 text-sm mt-1">{errors.title.message}</p>}
        </div>

        {/* Category */}
        <div>
          <label className="block text-sm font-medium mb-2">Category *</label>
          <select
            {...register('category')}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-emerald-500 outline-none"
          >
            <option value="infrastructure">Infrastructure</option>
            <option value="safety">Safety</option>
            <option value="environment">Environment</option>
            <option value="sanitation">Sanitation</option>
            <option value="transportation">Transportation</option>
            <option value="public_utility">Public Utility</option>
            <option value="other">Other</option>
          </select>
        </div>

        {/* Description */}
        <div>
          <label className="block text-sm font-medium mb-2">Description *</label>
          <textarea
            {...register('description')}
            rows={4}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-emerald-500 outline-none resize-none"
            placeholder="Describe the issue in detail..."
            onBlur={analyzeIssue}
          />
          {errors.description && <p className="text-red-400 text-sm mt-1">{errors.description.message}</p>}
        </div>

        {/* Location */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-2">Latitude</label>
            <input
              {...register('latitude', { valueAsNumber: true })}
              className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">Longitude</label>
            <input
              {...register('longitude', { valueAsNumber: true })}
              className="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg"
            />
          </div>
        </div>

        {/* AI Analysis */}
        {aiAnalysis && (
          <div className="glass-panel p-4 border-l-4 border-emerald-500">
            <div className="flex items-start space-x-3">
              <AlertTriangle className="w-5 h-5 text-emerald-400 mt-0.5" />
              <p className="text-sm text-slate-300">{aiAnalysis}</p>
            </div>
          </div>
        )}

        {/* Submit */}
        <button
          type="submit"
          disabled={isSubmitting}
          className="w-full btn-primary flex items-center justify-center space-x-2 disabled:opacity-50"
        >
          {isSubmitting ? (
            <span className="animate-spin">⟳</span>
          ) : (
            <>
              <Send className="w-4 h-4" />
              <span>Submit Report</span>
            </>
          )}
        </button>
      </form>
    </div>
  )
}
