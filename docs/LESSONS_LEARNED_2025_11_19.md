# Lessons Learned - November 19, 2025

## Critical Issues Encountered and Resolved

### 1. UI Authentication & Navigation Issues

#### Issue: User Stuck on Login Page After Successful Authentication
**Symptoms:**
- User logs in successfully with valid credentials
- JWT token returned from API
- But page stays on `/login` instead of redirecting to `/dashboard`

**Root Cause:**
- Using Next.js `router.push()` for navigation after login
- Client-side navigation doesn't trigger full page reload
- Dashboard layout checks `localStorage` for auth token immediately
- Race condition: dashboard mounts before `localStorage` is readable

**Solution:**
```typescript
// WRONG: Client-side navigation
router.push('/dashboard');

// CORRECT: Full page reload
window.location.href = '/dashboard';
```

**Files Changed:**
- `ui/app/(auth)/login/page.tsx`

**Checkpoint:** Always use `window.location.href` for post-authentication navigation to ensure localStorage is fully committed.

---

#### Issue: Dashboard Pages Not Loading After Login
**Symptoms:**
- After successful login redirect, dashboard shows loading spinner indefinitely
- User profile query doesn't execute
- Other dashboard pages inaccessible

**Root Cause:**
- ESLint rule `react-hooks/set-state-in-effect` violation
- Calling `setState` synchronously within `useEffect` body
- Causes cascading renders and unpredictable behavior

**Solution:**
```typescript
// WRONG: setState in useEffect body
useEffect(() => {
  const token = getAuthToken();
  if (!token) {
    router.replace('/login');
  } else {
    setIsReady(true); // ❌ Violates React rules
  }
}, [router]);

// CORRECT: Use useState initializer
const [hasValidToken] = useState(() => {
  const token = getAuthToken();
  if (!token) {
    setTimeout(() => router.replace('/login'), 0);
    return false;
  }
  return true;
});
```

**Files Changed:**
- `ui/app/(dashboard)/layout.tsx`

**Checkpoint:** 
- Never call `setState` directly in `useEffect` body
- Use `useState` initializer for one-time checks
- Schedule redirects with `setTimeout` to avoid sync render issues
- Enable and respect ESLint rules, especially `react-hooks/*`

---

### 2. CI/CD Pipeline Issues

#### Issue: E2E Tests Failing with 404 and Timeout (Exit Code 124)
**Symptoms:**
```
curl: (22) The requested URL returned error: 404
Error: Process completed with exit code 124.
```

**Root Cause:**
- E2E workflow health check using wrong endpoint path
- Checking `/health` instead of `/api/v1/health`
- 30-second timeout expires waiting for non-existent endpoint

**Solution:**
```yaml
# WRONG
timeout 30 bash -c 'until curl -f http://localhost:8080/health; do sleep 1; done'

# CORRECT
timeout 30 bash -c 'until curl -f http://localhost:8080/api/v1/health; do sleep 1; done'
```

**Files Changed:**
- `.github/workflows/e2e-tests.yml`

**Checkpoint:** Always verify API endpoint paths match actual route configuration before setting up health checks.

---

#### Issue: Redis Consumer Group Mismatch in CI
**Symptoms:**
```
NOGROUP No such consumer group 'message_processors' for key 'events:11155111'
```

**Root Cause:**
- Global `ENVIRONMENT=development` set in CI workflow
- Overrode test-specific environment settings
- Tests created consumer groups for production chains (1, 42161, 10, 8453)
- But message-processor running with `ENVIRONMENT=development` expected Sepolia (11155111)

**Solution:**
1. Remove global `ENVIRONMENT=development` from CI workflow
2. Let each test set its own environment
3. Ensure consumer group creation matches the environment's chain configuration

**Files Changed:**
- `.github/workflows/ci.yml`
- `tests/e2e_tests.rs`

**Checkpoint:** 
- Avoid setting global environment variables that override test-specific configs
- Consumer groups must match the chains defined in the active environment
- Test setup must mirror production environment configuration

---

#### Issue: Stale Cargo Build Cache
**Symptoms:**
- Code changes not reflected in CI builds
- Old binaries being executed despite source file changes

**Root Cause:**
- Cache key only included `Cargo.lock` hash
- Source file changes didn't invalidate cache

**Solution:**
```yaml
# WRONG
key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

# CORRECT
key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/*.rs') }}
```

**Files Changed:**
- `.github/workflows/ci.yml`
- `.github/workflows/digitalocean-deploy.yml`

**Checkpoint:** Always include source file hashes in build cache keys to ensure cache invalidation on code changes.

---

### 3. Deployment Issues

#### Issue: Container Name Conflicts During Deployment
**Symptoms:**
```
Error response from daemon: Conflict. The container name "/ethhook-demo-receiver" is already in use
```

**Root Cause:**
- Lingering containers from previous deployments
- `docker compose up` fails when container names are taken

**Solution:**
```bash
# Add before deployment
docker rm -f ethhook-admin-api ethhook-message-processor ethhook-webhook-delivery ethhook-event-ingestor ethhook-demo-receiver ethhook-redis ethhook-postgres || true
```

**Files Changed:**
- `.github/workflows/digitalocean-deploy.yml`

**Checkpoint:** Always force remove containers before redeployment to avoid name conflicts.

---

### 4. Production Database Issues

#### Issue: Connection Pool Exhaustion
**Symptoms:**
- Services showing unhealthy status
- Database connection timeouts
- Pool size exceeded errors

**Root Cause:**
- Default connection pool size too small for production load
- Potential connection leaks in long-running services

**Recommended Solution (Not Yet Implemented):**
1. Increase `DATABASE_MAX_CONNECTIONS` in production environment
2. Review connection handling in all services
3. Ensure connections are properly released after use
4. Add connection pool metrics to monitoring

**Checkpoint:** 
- Set appropriate connection pool sizes for production (start with 20-50)
- Monitor connection pool usage
- Implement proper connection lifecycle management
- Add alerts for pool exhaustion

---

## Configuration Checklist for Future Development

### Environment Configuration
- [ ] Ensure development environment uses only Sepolia (chain_id: 11155111)
- [ ] Ensure production environment uses mainnet chains (1, 42161, 10, 8453)
- [ ] Avoid global environment variables that override test-specific settings
- [ ] Consumer groups must match chain IDs for the active environment

### CI/CD Pipeline
- [ ] Cache keys include both dependency locks AND source file hashes
- [ ] Health check endpoints match actual API routes (`/api/v1/health`)
- [ ] Deployment scripts force remove old containers before starting new ones
- [ ] Each test job sets its own environment variables

### Frontend Authentication
- [ ] Use `window.location.href` for post-login navigation (not `router.push`)
- [ ] Never call `setState` directly in `useEffect` body
- [ ] Use `useState` initializer for one-time synchronous checks
- [ ] Enable and respect all ESLint rules, especially React hooks rules

### Backend Services
- [ ] Health endpoints accessible at documented paths
- [ ] Connection pool sizes appropriate for load
- [ ] All database connections properly released
- [ ] Consumer groups pre-created for all configured chains

### Docker & Deployment
- [ ] Force remove containers before deployment (`docker rm -f`)
- [ ] Environment files properly configured for target environment
- [ ] Services properly health-checked before marking deployment complete
- [ ] Redis streams and consumer groups initialized correctly

---

## Quick Reference Commands

### Check Production Status
```bash
ssh root@104.248.15.178 "cd ~/ethhook && docker compose -f docker-compose.prod.yml ps"
```

### Test Production Login
```bash
curl -X POST http://104.248.15.178:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"demo@ethhook.com","password":"demo123"}'
```

### Check Redis Consumer Groups
```bash
redis-cli XINFO GROUPS events:11155111
```

### Force Restart All Services
```bash
docker compose -f docker-compose.prod.yml down
docker rm -f $(docker ps -aq) || true
docker compose -f docker-compose.prod.yml up -d
```

---

## Related Documentation
- `ADMIN_LOGIN_FIXED.md` - Admin authentication fixes
- `UI_MIGRATION_COMPLETE.md` - UI modernization details
- `E2E_TESTS_FIXED.md` - E2E test setup and fixes
- `DEPLOYMENT_QUICKSTART.md` - Deployment procedures

---

## Prevention Strategies

1. **Pre-Push Hooks**: Already in place - catches most issues before CI
2. **ESLint Configuration**: Enforces React best practices
3. **Type Checking**: TypeScript catches interface mismatches
4. **Integration Tests**: Verify environment configuration matches expectations
5. **Health Checks**: Ensure all services respond correctly before deployment

---

**Last Updated:** November 19, 2025
**Total Issues Resolved:** 7 critical issues
**Services Affected:** UI (Next.js), CI Pipeline, Deployment, Redis, Backend Services
