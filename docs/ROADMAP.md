# Roadmap

## Phase 1: Foundation
- [x] Project scaffolding
- [x] Basic API endpoints
- [x] React frontend shell
- [x] Docker setup
- [x] CI/CD pipelines
- [x] Database integration
- [x] User registration and login
- [x] Full issue CRUD operations
- [x] File uploads (photos)

## Phase 2: Intelligence
- [ ] WebAssembly analytics module
- [ ] Browser-side text analysis
- [ ] Category auto-suggestion
- [ ] Priority scoring
- [ ] Duplicate detection
- [ ] Real-time notifications

## Phase 3: Community
- [ ] Mobile PWA
- [ ] LINE integration
- [x] Multi-language support (EN/TH/JA selector + fallback)
- [ ] Public API
- [ ] Open data export

## Phase 4: Ecosystem
- [ ] Plugin system
- [ ] Third-party integrations
- [ ] White-label option
- [ ] Advanced analytics

## Contributing

Have an idea? Open an issue or discussion. We listen.

## Handoff Log

### 2026-05-20

Completed in this continuation:
- Fixed frontend integration after `useIssues` return-shape change (`{ issues, meta }`) by updating map/dashboard consumers.
- Added `useIssue(id)` hook for issue detail fetch use-cases.
- Updated issue category token usage to `publicutility` across report/map UI for backend consistency.
- Updated API issue creation route to optionally extract `reporter_id` from `Authorization: Bearer <jwt>` when provided.
- Updated API server startup to use configured CORS origin list and configured server port.
- Fixed Rust route tests to match `create_issue` signature with request headers.

Validation run:
- `apps/web`: `npm run typecheck` and `npm run build` passed.
- `apps/api`: `cargo check` and `cargo test` passed (11 tests).

Suggested next tasks:
1. Decide whether `/dashboard` and `/map` should remain public or be re-protected with auth.
2. Implement file upload for issue photos (currently URL list only).
3. Improve role policy for comment creation payload (`author_id` currently client-provided).

### 2026-05-20 (Issue Detail Completion)

Completed in this continuation:
- Added issue detail frontend page at `/issues/:id` with:
  - issue metadata and description
  - comments list + authenticated comment submission
  - status history timeline
- Wired navigation to detail page from:
  - report flow (auto-redirect after successful create)
  - dashboard recent activity cards
  - map popup and map sidebar list
- Added frontend API models and hooks:
  - `IssueComment`, `IssueStatusHistoryEntry`, `CreateCommentRequest`
  - `useIssueComments(id)` and `useIssueHistory(id)`
- Normalized dashboard category color map for `publicutility`.

Validation run:
- `apps/web`: `npm run typecheck` and `npm run build` passed.
- `apps/api`: `cargo test` passed (11 tests).

Suggested next tasks:
1. Implement file upload for issue photos (backend storage + frontend uploader UI).
2. Harden comment POST API to derive `author_id` from JWT server-side (ignore client-sent author id).
3. Decide and enforce auth policy for `/dashboard` and `/map`.

### 2026-05-20 (Uploads + Comment Auth Hardening)

Completed in this continuation:
- Added image upload backend endpoint `POST /api/v1/uploads` (multipart field: `file`), max 10MB, image-only validation.
- Added static file serving for uploaded media under `/uploads/*` via `ServeDir`.
- Added frontend upload UX on report form (multi-image upload, preview, remove-before-submit, and media URL submission with issue create).
- Hardened comment creation security:
  - `author_id` is now derived from JWT claims server-side.
  - reporters cannot force internal comments.
  - empty comments are rejected.
- Updated API models and frontend types to match the new comment payload contract.

Validation run:
- `apps/api`: `cargo test` passed (11 tests).
- `apps/web`: `npm run typecheck` and `npm run build` passed.

Suggested next tasks:
1. Add backend image cleanup strategy (orphaned uploads, delete-on-issue-delete policy).
2. Decide auth policy for `/dashboard` and `/map` and enforce consistently in router.
3. Add integration tests for upload + comment auth hardening paths.

### 2026-05-20 (Blank Screen Hardening + UI Tone Refresh)

Completed in this continuation:
- Hardened auth bootstrap against corrupted browser storage:
  - guarded JSON parsing for `civic_user`
  - sanitized invalid `civic_token` values (`undefined`/`null`)
- Added `AppErrorBoundary` wrapper to prevent white/blank screen on unhandled runtime errors and provide a recovery reload action.
- Refined UI tone to a calmer, less saturated palette (blue-slate accents) across shared styles and key entry pages:
  - shared `index.css` theme tokens/components
  - header/navigation (`Layout`)
  - home hero/feature accents
  - login/register forms and buttons

Validation run:
- `apps/web`: `npm run typecheck` and `npm run build` passed.

Suggested next tasks:
1. Add a lightweight browser E2E smoke test to catch runtime blank-screen regressions automatically.
2. Apply the same calm color cleanup to dashboard/map/detail micro-elements for full visual consistency.

### 2026-05-20 (Tailwind Pipeline Restore)

Completed in this continuation:
- Root-caused unstyled/blank-feeling UI to missing Tailwind/PostCSS pipeline configs in `apps/web`.
- Added:
  - `apps/web/tailwind.config.cjs`
  - `apps/web/postcss.config.cjs`
- Verified CSS generation now contains compiled utility classes (not raw `@tailwind` directives).
- Restarted Vite dev server and verified:
  - `GET /` returns `200`
  - `GET /src/index.css` returns `200` with compiled Tailwind output

Validation run:
- `apps/web`: `npm run typecheck` passed.

Suggested next tasks:
1. Add CI guard to fail build if compiled CSS still contains raw `@tailwind` directives.

### 2026-05-20 (Multi-language UI + Map Location Picker)

Completed in this continuation:
- Added frontend i18n foundation:
  - `LanguageProvider` + persisted language preference (`civic_language`)
  - translation dictionaries with EN/TH coverage and JA fallback support
  - token formatters for category/status/priority labels
- Added language selector to both desktop and mobile navigation.
- Localized key screens and flows:
  - layout/home/login/register/report/dashboard/map/issue-detail UI labels
  - toast messages and common action text
- Improved report location handling:
  - selectable map marker (click map or drag pin)
  - current-location autofill via browser geolocation
  - place search (geocoding) + reverse geocode to address
  - submitted issue now includes optional location address
- Improved live map behavior:
  - auto-fit viewport to filtered issue set
  - auto-focus selected issue
  - translated status/category/priority labels in sidebar and popup

Validation run:
- `apps/web`: `npm run typecheck` and `npm run build` passed.

Suggested next tasks:
1. Add language-aware validation/error mapping for all backend error payloads (not only frontend defaults).
2. Add integration/e2e test for report location picker (map click, geolocation, search, submit payload).
3. Consider extracting map/geocoding logic into reusable hooks/components to reduce page complexity.

### 2026-05-20 (Trust Verification + OTP + High-volume Triage + Map 24h)

Completed in this continuation:
- Implemented trust metadata on issues:
  - `verification_score`, `verification_state`, `duplicate_of`, `corroboration_count`, `triage_score`
  - `VerificationState` enum (`trusted`, `needs_review`, `suspicious`)
- Enforced photo evidence on issue creation:
  - `POST /api/v1/issues` now rejects requests without at least 1 photo (`400`)
- Added duplicate clustering at create-time:
  - candidate prefilter by active statuses + 30 days + bbox ±0.002
  - duplicate decision by rules engine (`distance <= 100m`, title similarity `>= 0.75`)
  - root issue corroboration is incremented and root trust/triage is recalculated
- Added SQL-level triage/filter/pagination for issue listing:
  - supports `status/category/priority/verification_state`
  - supports sort by `triage|created_at|updated_at|priority`
  - uses DB `LIMIT/OFFSET` and SQL `COUNT(*)`
- Added OTP phone verification flow:
  - `POST /api/v1/auth/otp/request`
  - `POST /api/v1/auth/otp/verify`
  - OTP challenge/token tables and indexes
  - optional phone verification token accepted on `POST /api/v1/issues` (`otp_phone` + `otp_token`)
- Added/updated frontend types and report flow:
  - issue model includes trust/duplicate/triage fields
  - report form enforces at least 1 uploaded image before submit
  - optional OTP request/verify UI integrated in report form
- Updated map time display to 24-hour format on map page only:
  - sidebar + popup now show created/updated with `hour12: false`
  - still uses user locale/timezone
- Added i18n keys for OTP and map created/updated labels (EN/TH, JA falls back to EN).

Validation run:
- `apps/web`: `npm run typecheck` passed.
- `apps/web`: `npm run build` passed.
- `apps/api`: direct `cargo` tool not available on host, and Docker daemon was not running (`dockerDesktopLinuxEngine` not found), so backend tests were not executable in this environment.

Suggested next tasks:
1. Run `cargo test --workspace` in an environment with Rust toolchain (or start Docker daemon and run tests in container) to confirm backend compile/tests.
2. Add backend integration tests for OTP flow and duplicate/triage ranking behavior.
3. Add admin moderation view filters for `verification_state` and `triage_score` to operationalize review queue.

### 2026-05-20 (Usability Upgrade + OTP Country Selector + Admin Queue UI)

Completed in this continuation:
- Improved OTP UX on report form for real-world phone handling:
  - added country selector with dial code (`TH/US/JP/SG/MY`)
  - local number input now normalizes to E.164 before OTP request/verify
  - upgraded OTP input to 6 separate numeric boxes with auto-focus and paste support
  - added resend cooldown state and clearer helper text
- Added admin operational queue page (`/admin`) focused on high-volume handling:
  - summary cards for active/needs-review/suspicious/high-triage counts
  - server-backed queue filters (`status`, `verification_state`, `sort`) and local search
  - per-issue quick status update action with reason support for backward/reopen transitions
  - visibility guard: page shows restricted-state UX for non-admin/non-responder users
- Updated navigation and routing:
  - added `Admin Queue` nav item for `admin`/`responder` users
  - wired new route in app router
- Extended web API/hook types:
  - added `IssueFilters`, `UpdateIssueRequest`
  - updated `useIssues` to support filter/sort query params
  - added `updateIssueStatus()` helper
- Added i18n strings (EN/TH) for:
  - OTP country-aware UX messages
  - admin queue labels, filters, update actions, and restricted view states

Validation run:
- `apps/web`: `npm run typecheck` passed.
- `apps/web`: `npm run build` passed.

Suggested next tasks:
1. Add country list + formatting powered by a phone metadata library (e.g., libphonenumber) for stricter validation edge-cases.
2. Add OTP retry/lockout UX hints that mirror backend limits (max attempts, expiry countdown).
3. Add admin bulk actions (multi-select + batch status update) for faster triage operations.

### 2026-05-20 (OTP/Admin Edge-case Polish)

Completed in this continuation:
- OTP form hardening:
  - changing country or local number now resets verification token and clears OTP digits
  - editing/pasting OTP code clears prior verified state to prevent stale verification reuse
  - country/phone changes reset resend cooldown for better operator flow
- Admin queue draft-state bug fix:
  - fixed status draft initialization so editing reason does not silently revert status to `reported`

Validation run:
- `apps/web`: `npm run typecheck` passed.
- `apps/web`: `npm run build` passed.

Suggested next tasks:
1. Add component-level tests for OTP digit input/paste behavior and admin draft-state transitions.
2. Add server-side phone normalization parity tests for E.164 and legacy input handling.

### 2026-05-20 (UI Polish Smoke Check)

Completed in this continuation:
- Ran the Vite dev server and visually checked the main UI surfaces in browser:
  - report flow
  - admin restricted state
  - live map
  - dashboard loading state
- Improved shared visual system:
  - replaced the flat one-color app background with a restrained civic grid/gradient backdrop
  - tightened shared panel radius/shadow treatment
  - added reusable `form-section`, `form-label`, and `form-control` styles
- Polished top navigation:
  - removed card-like rounded header styling
  - added cleaner sticky header treatment and calmer active nav state
- Polished report UX:
  - grouped issue detail fields into a clear form section
  - standardized input/select/textarea focus states
  - improved location, OTP, and photo upload panel consistency

Validation run:
- Browser smoke check completed against `http://127.0.0.1:5173`.
- `apps/web`: `npm run typecheck` passed.
- `apps/web`: `npm run build` passed.

Suggested next tasks:
1. Add a visual regression/smoke test for `/report`, `/admin`, `/map`, and `/dashboard`.
2. Consider code-splitting route chunks to reduce the current Vite bundle size warning.
