# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Civic Sentinel, please report it responsibly:

1. **Do NOT** create a public GitHub issue
2. Email the maintainer directly at: rtchanaphon@gmail.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a detailed response within 7 days.

---

## Security Measures Implemented

### Authentication & Authorization

✅ **JWT-based Authentication**
- Access tokens with configurable expiration (default: 24 hours)
- Refresh token rotation to prevent token theft
- Argon2 password hashing (memory: 64MB, iterations: 3, parallelism: 4)
- Automatic token refresh on 401 responses

✅ **Phone OTP Verification**
- 6-digit OTP with SHA-256 hashing
- 5-minute expiration window
- Maximum 5 attempts per challenge
- Consumed tokens cannot be reused

✅ **Role-Based Access Control**
- Four roles: Admin, Responder, Reporter, Viewer
- Protected routes require authentication middleware
- Role checks on sensitive operations (update, delete)

### Input Validation & Injection Prevention

✅ **SQL Injection Protection**
- Enum-based filtering for list queries (status, category, priority, verification_state)
- All filter values validated through strict enum parsing
- Sort parameter uses whitelist matching only
- No raw user input concatenated into SQL queries

✅ **Request Validation**
- Pagination limits clamped (max 100 items per page)
- Enum parsing rejects invalid values
- UUID validation for all ID parameters

⚠️ **Pending Improvements:**
- [ ] Length limits on title/description fields
- [ ] Comprehensive file upload validation (MIME type, size, content inspection)
- [ ] Rate limiting on OTP requests (currently unlimited challenges)

### Rate Limiting & DoS Prevention

✅ **Token Bucket Rate Limiter**
- Configurable limits (default: 100 requests per 60 seconds)
- Per-IP tracking using X-Forwarded-For or X-Real-IP headers
- Shared instance in AppState (prevents memory leak)
- Automatic cleanup of old buckets every 5 minutes

✅ **Connection Limits**
- HTTP/2 multiplexing via Axum
- Graceful degradation on high load

⚠️ **Pending Improvements:**
- [ ] Request body size limits (currently unlimited)
- [ ] Connection pooling for database
- [ ] Distributed rate limiting (Redis) for multi-instance deployments

### Headers & Transport Security

✅ **Security Headers Middleware**
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security` (configure reverse proxy)

✅ **CORS Configuration**
- Configurable allowed origins
- Rejects requests from unauthorized domains

⚠️ **Pending Improvements:**
- [ ] Content-Security-Policy header
- [ ] CSRF protection tokens
- [ ] Referrer-Policy header

### File Uploads

⚠️ **Current State:**
- Accepts multipart/form-data uploads
- Stores files in local `UPLOAD_DIR`
- Serves files directly via `/uploads` route

⚠️ **Critical Gaps (High Priority):**
- [ ] File type whitelist (image MIME types only)
- [ ] File size limits (e.g., 10MB max)
- [ ] Filename sanitization (prevent path traversal)
- [ ] MIME type validation (not just extension)
- [ ] Content-Disposition: attachment header
- [ ] Optional: malware scanning integration

### Database Security

✅ **Connection Security**
- Supports local SQLite and remote Turso (TLS)
- Auth tokens for Turso connections
- Prepared statements for all queries

✅ **Data Protection**
- Soft delete for issues (preserves audit trail)
- Password hashes never logged or exposed
- Refresh tokens stored with SHA-256 hash

⚠️ **Pending Improvements:**
- [ ] Database connection pooling
- [ ] Encrypted backups
- [ ] Field-level encryption for sensitive data (e.g., phone numbers)

### Logging & Monitoring

✅ **Tracing Infrastructure**
- Structured logging via `tracing` crate
- Configurable log levels (RUST_LOG env var)
- Request/response logging (excluding sensitive headers)

⚠️ **Pending Improvements:**
- [ ] Request correlation IDs
- [ ] Failed authentication attempt logging
- [ ] Anomaly detection alerts
- [ ] Log aggregation (e.g., Loki, CloudWatch)

---

## Known Issues & Mitigations

### 1. JWT Secret Default Value
**Severity:** Critical  
**Status:** Documented, validation pending  
**Issue:** Default JWT_SECRET in .env.example is weak  
**Mitigation:** Startup validation will reject default/weak secrets (planned)  
**Workaround:** Manually verify JWT_SECRET is changed before deployment

### 2. Refresh Tokens in localStorage
**Severity:** Medium  
**Status:** Architectural decision  
**Issue:** XSS can steal tokens from localStorage  
**Mitigation:** Using httpOnly cookies for refresh tokens is more secure but complex  
**Recommendation:** Implement Content-Security-Policy to reduce XSS risk

### 3. CORS Wildcard Support
**Severity:** Medium  
**Status:** Configurable  
**Issue:** Allows `CORS_ORIGINS=*` which accepts all origins  
**Mitigation:** Document that wildcard should NEVER be used in production  
**Recommendation:** Reject wildcard in production mode (planned)

### 4. No CSRF Protection
**Severity:** Medium  
**Status:** Planned  
**Issue:** State-changing requests lack CSRF tokens  
**Mitigation:** CORS restrictions limit exposure  
**Recommendation:** Implement double-submit cookie or synchronizer token pattern

### 5. Rate Limiter Bypass via Multiple IPs
**Severity:** Low  
**Status:** By design  
**Issue:** Distributed DoS can bypass per-IP rate limits  
**Mitigation:** Deploy behind CDN (Cloudflare) or WAF  
**Recommendation:** Add global rate limits or connection limits

---

## Secure Configuration Checklist

### Production Deployment

**Required:**
- [ ] Set unique JWT_SECRET (32+ characters, random)
- [ ] Use HTTPS (reverse proxy or platform SSL)
- [ ] Restrict CORS_ORIGINS to specific domains (no `*`)
- [ ] Enable DATABASE_URL encryption (use Turso with auth token)
- [ ] Set RUST_LOG=info (not debug)
- [ ] Configure UPLOAD_DIR with limited permissions (0755)
- [ ] Enable firewall rules (only ports 80/443 exposed)

**Recommended:**
- [ ] Use environment variables (not .env files) in production
- [ ] Enable automatic security updates
- [ ] Set up log monitoring and alerting
- [ ] Implement database backup strategy
- [ ] Configure fail2ban or similar intrusion prevention
- [ ] Add Content-Security-Policy headers
- [ ] Use HTTP/2 and enable HSTS

**Optional:**
- [ ] Deploy behind CDN (Cloudflare, CloudFront)
- [ ] Enable WAF (Web Application Firewall)
- [ ] Implement penetration testing schedule
- [ ] Set up vulnerability scanning (Dependabot, Snyk)

---

## Dependencies & Supply Chain

### Rust Dependencies (Cargo)
- Managed via `Cargo.toml` with version pinning
- Regular updates via `cargo update`
- Security advisories checked via `cargo audit` (recommended)

### Frontend Dependencies (npm)
- Managed via `package.json` with version pinning
- Regular updates via `npm audit fix`
- Lockfile (`package-lock.json`) committed to repository

### Recommended Tools
```bash
# Rust security audit
cargo install cargo-audit
cargo audit

# Frontend security audit  
npm audit

# Check for outdated packages
cargo outdated
npm outdated
```

---

## Incident Response Plan

If a security vulnerability is exploited in your deployment:

1. **Immediate Actions:**
   - Isolate affected systems
   - Rotate all JWT secrets
   - Revoke all refresh tokens (delete from database)
   - Review access logs for suspicious activity

2. **Investigation:**
   - Identify attack vector
   - Assess data exposure
   - Check database for unauthorized modifications

3. **Remediation:**
   - Apply patches or hotfixes
   - Restore from clean backups if compromised
   - Notify affected users (if applicable)

4. **Prevention:**
   - Update security policies
   - Add monitoring rules
   - Schedule security review

---

## Security Roadmap

### Q3 2026 (Current)
- [x] SQL injection protection (enum-based filtering)
- [x] Rate limiter memory leak fix
- [ ] File upload validation
- [ ] JWT secret validation on startup
- [ ] Request body size limits

### Q4 2026
- [ ] CSRF protection
- [ ] Database connection pooling
- [ ] Content-Security-Policy headers
- [ ] Integration tests for auth flows
- [ ] Penetration testing

### 2027
- [ ] Field-level encryption
- [ ] Advanced anomaly detection
- [ ] Security audit by third party
- [ ] Bug bounty program (if public deployment)

---

## Contact

For security concerns: rtchanaphon@gmail.com  
For general issues: [GitHub Issues](https://github.com/FirstPrinciples-Sun/civic-sentinel/issues)

---

**Last updated:** July 3, 2026
