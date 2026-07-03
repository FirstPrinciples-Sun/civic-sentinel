# Code Review Checklist

This checklist ensures code quality, security, and consistency across the Civic Sentinel codebase.

---

## General Requirements

### Before Submitting PR
- [ ] Code compiles without errors: `cargo clippy -D warnings`
- [ ] All tests pass: `cargo test --workspace`
- [ ] Frontend builds: `npm run build`
- [ ] Code is formatted: `cargo fmt` and `npm run lint`
- [ ] No debugging code left (console.log, dbg!, println!)
- [ ] Branch is up to date with `develop`
- [ ] PR description explains what and why

### Documentation
- [ ] Public functions have doc comments
- [ ] Complex logic includes inline comments
- [ ] README updated if user-facing changes
- [ ] CHANGELOG.md updated with changes
- [ ] API changes documented (if applicable)

---

## Backend (Rust)

### Code Quality
- [ ] No `.unwrap()` or `.expect()` in production code (use `?` operator or proper error handling)
- [ ] Error types are descriptive (use `thiserror` or custom types)
- [ ] No `panic!` in request handlers
- [ ] Constants extracted (no magic numbers/strings)
- [ ] Code is DRY (no duplicate logic)
- [ ] Functions are single-purpose (max ~50 lines)

### Performance
- [ ] Database queries are efficient (no N+1 queries)
- [ ] Proper indexes exist for queried columns
- [ ] Avoid cloning large structs unnecessarily
- [ ] Use `&str` instead of `String` where possible
- [ ] Async functions don't block (no `.await` in loops without consideration)

### Security
- [ ] No SQL injection risk (use parameterized queries or validated enums)
- [ ] User input is validated (length, format, range)
- [ ] Authentication required for protected routes
- [ ] Authorization checks (role validation) on sensitive operations
- [ ] Secrets not logged or exposed in errors
- [ ] File paths sanitized (no path traversal)
- [ ] Rate limiting applied where appropriate

### Database
- [ ] Migrations are reversible (if applicable)
- [ ] New columns have sensible defaults
- [ ] Foreign keys defined correctly
- [ ] Indexes added for filtered/sorted columns
- [ ] Soft delete used for audit trail

### Testing
- [ ] Unit tests for business logic
- [ ] Integration tests for API endpoints
- [ ] Error cases tested (not just happy path)
- [ ] Edge cases covered (empty input, max values, etc.)
- [ ] Mock external dependencies

---

## Frontend (React/TypeScript)

### Code Quality
- [ ] TypeScript strict mode passes: `npm run typecheck`
- [ ] No `any` types (use proper interfaces/types)
- [ ] Components are single-purpose
- [ ] Props have explicit types
- [ ] Custom hooks extracted for reusable logic
- [ ] Error boundaries handle component errors

### Performance
- [ ] Expensive calculations memoized (`useMemo`, `useCallback`)
- [ ] Avoid unnecessary re-renders (React.memo, proper dependencies)
- [ ] Large lists use virtualization (if >100 items)
- [ ] Images lazy loaded or optimized
- [ ] Code splitting for routes (`React.lazy`)

### Security
- [ ] User input sanitized before display (prevent XSS)
- [ ] Tokens stored securely (currently localStorage, document if changed)
- [ ] API errors don't leak sensitive info
- [ ] CSRF tokens included (when implemented)
- [ ] External links use `rel="noopener noreferrer"`

### UX/Accessibility
- [ ] Loading states displayed during async operations
- [ ] Error messages are user-friendly
- [ ] Forms have validation feedback
- [ ] Keyboard navigation works (Tab, Enter, Esc)
- [ ] ARIA labels on interactive elements
- [ ] Color contrast meets WCAG AA (min 4.5:1)
- [ ] Focus indicators visible

### Testing
- [ ] Unit tests for utilities and hooks
- [ ] Component tests for critical UI
- [ ] Form validation tested
- [ ] Error handling tested

---

## API Design

### REST Conventions
- [ ] Proper HTTP methods (GET/POST/PATCH/DELETE)
- [ ] Status codes are semantic (200, 201, 400, 401, 404, 500, etc.)
- [ ] JSON responses follow consistent structure
- [ ] Pagination for list endpoints (page, limit, total)
- [ ] Filtering uses query parameters
- [ ] Versioned endpoints (`/api/v1/...`)

### Request Validation
- [ ] Required fields enforced
- [ ] Optional fields have defaults
- [ ] Input lengths validated
- [ ] Enum values validated
- [ ] UUIDs validated
- [ ] Dates in standard format (ISO 8601)

### Response Format
- [ ] Success responses include `success: true`
- [ ] Error responses include `error` field with message
- [ ] List responses include metadata (`total`, `page`, etc.)
- [ ] Timestamps in ISO 8601 format
- [ ] No sensitive data exposed

---

## Security Review

### Authentication
- [ ] JWT tokens validated on protected routes
- [ ] Token expiration enforced
- [ ] Refresh token rotation implemented
- [ ] Password strength requirements met
- [ ] Rate limiting on login/register endpoints

### Authorization
- [ ] User can only access their own data
- [ ] Admin checks enforced on privileged operations
- [ ] Role-based permissions verified
- [ ] No privilege escalation possible

### Data Protection
- [ ] Passwords hashed (Argon2)
- [ ] Sensitive data encrypted at rest (if applicable)
- [ ] Audit logs for sensitive operations
- [ ] Soft delete preserves records
- [ ] No PII in logs

### Input Validation
- [ ] All user input validated
- [ ] File uploads restricted (type, size)
- [ ] SQL injection prevented
- [ ] XSS prevented (output encoding)
- [ ] CSRF tokens validated (when implemented)

---

## Database Review

### Schema Changes
- [ ] Migration tested locally
- [ ] No breaking changes (or properly handled)
- [ ] New columns nullable or have defaults
- [ ] Indexes added for performance
- [ ] Foreign keys maintain referential integrity

### Query Performance
- [ ] No full table scans (explain query plan)
- [ ] Proper indexes used
- [ ] Limit clauses on large result sets
- [ ] Avoid N+1 queries (use joins)
- [ ] Aggregate functions used efficiently

---

## Testing Standards

### Unit Tests
- [ ] Test files colocated with source (Rust) or in `__tests__` (TS)
- [ ] Test names describe behavior ("should reject invalid JWT")
- [ ] Arrange-Act-Assert pattern
- [ ] Mock external dependencies
- [ ] Test both success and failure cases

### Integration Tests
- [ ] Test full request/response cycle
- [ ] Use test database (not production)
- [ ] Clean up test data after run
- [ ] Test authentication flow end-to-end
- [ ] Test role-based access control

### Test Coverage
- [ ] Critical paths have tests (auth, payments, data mutations)
- [ ] Utility functions have unit tests
- [ ] Edge cases covered
- [ ] Security-critical code tested
- [ ] Aim for 70%+ coverage on core logic

---

## Performance Review

### Backend
- [ ] Response times < 200ms for simple queries
- [ ] Response times < 1s for complex operations
- [ ] Database queries optimized (EXPLAIN ANALYZE)
- [ ] No synchronous blocking in async code
- [ ] Connection pooling used (when implemented)

### Frontend
- [ ] Initial page load < 3s
- [ ] Time to interactive < 5s
- [ ] Bundle size < 500KB (gzipped)
- [ ] Lighthouse score > 80
- [ ] No console errors in production build

---

## Documentation Review

### Code Documentation
- [ ] Public API functions have doc comments
- [ ] Complex algorithms explained
- [ ] "Why" not just "what" documented
- [ ] Examples provided for non-obvious usage
- [ ] TODOs tracked (with issue links if possible)

### User Documentation
- [ ] README updated for new features
- [ ] Configuration options documented
- [ ] Migration guides for breaking changes
- [ ] Examples provided for common use cases
- [ ] Troubleshooting section updated

---

## Git Hygiene

### Commit Messages
- [ ] Follow conventional commits format: `type(scope): description`
  - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
  - Example: `feat(auth): add phone OTP verification`
- [ ] First line < 72 characters
- [ ] Body explains "why" not "what"
- [ ] References related issues (#123)

### Branch Strategy
- [ ] Branch from `develop` (not `main`)
- [ ] Branch name: `feature/short-description` or `fix/bug-name`
- [ ] PR targets `develop` (not `main`)
- [ ] No merge conflicts
- [ ] Squash commits if messy history

---

## Reviewer Responsibilities

### Code Review Goals
1. **Correctness:** Does it work as intended?
2. **Security:** Are there vulnerabilities?
3. **Maintainability:** Is it readable and extendable?
4. **Performance:** Are there obvious bottlenecks?
5. **Testing:** Is it adequately tested?

### Review Etiquette
- Be constructive and respectful
- Explain "why" for requested changes
- Distinguish between blocking issues and suggestions
- Acknowledge good patterns/solutions
- Approve when criteria met (don't nitpick style)

### Blocking Issues (Must Fix)
- Security vulnerabilities
- Breaking changes without migration path
- Failing tests
- Significant performance degradation
- Violates architecture principles

### Non-Blocking Suggestions
- Style preferences (if code is already formatted)
- Alternative approaches (without clear benefit)
- Refactoring opportunities (can be separate PR)
- Documentation improvements (can be follow-up)

---

## Automated Checks (CI)

The following must pass before merge:

### Backend
- `cargo fmt --check` (formatting)
- `cargo clippy -- -D warnings` (linting)
- `cargo test --workspace` (tests)
- `cargo audit` (security advisories)

### Frontend
- `npm run lint` (ESLint)
- `npm run typecheck` (TypeScript)
- `npm run test` (Vitest)
- `npm audit` (security advisories)

### Both
- Build succeeds
- No merge conflicts
- PR approved by maintainer

---

## Checklist Usage

**For Contributors:**
- Copy relevant sections to your PR description
- Check off items as you complete them
- Note any deviations with explanation

**For Reviewers:**
- Use as guide during review
- Flag unchecked critical items
- Request changes or approve based on completion

---

**Last updated:** July 3, 2026
