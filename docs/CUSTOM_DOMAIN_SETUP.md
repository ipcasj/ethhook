# Custom Domain Setup for EthHook on Railway

**Goal**: Use your own domain (e.g., `ethhook.io`) instead of Railway's default URLs

**Time**: 20-30 minutes
**Cost**: $10-15/year for domain
**Difficulty**: Easy

---

## Why Custom Domain?

### Railway Default URLs (not branded):
- ❌ `https://admin-api-production-a4b3.up.railway.app`
- ❌ `https://leptos-portal-production-c5d4.up.railway.app`
- ❌ Unprofessional, hard to remember

### Your Custom Domain (branded):
- ✅ `https://api.ethhook.io`
- ✅ `https://app.ethhook.io`
- ✅ Professional, easy to remember

---

## Step 1: Choose and Buy Domain (10 minutes)

### Recommended Registrars

#### Option A: Cloudflare (Best Overall) ⭐
- **Price**: $8-10/year
- **Pros**: Cheapest, includes DDoS protection, easy DNS
- **Cons**: Credit card required
- **URL**: https://www.cloudflare.com/products/registrar/

#### Option B: Namecheap (Most Popular)
- **Price**: $10-12/year
- **Pros**: Good UI, lots of TLDs, privacy protection
- **Cons**: Slightly more expensive
- **URL**: https://www.namecheap.com/

#### Option C: Google Domains
- **Price**: $12/year
- **Pros**: Simple, trusted, good integration
- **Cons**: More expensive
- **URL**: https://domains.google/

### Domain Name Suggestions

**Available (probably):**
- `ethhook.io` - Tech-focused, short
- `ethhook.dev` - Developer-focused
- `ethhook.app` - Modern, app-focused
- `ethwebhook.com` - Descriptive
- `hooketh.io` - Creative twist

**Check Availability**:
1. Go to registrar website
2. Search for your desired domain
3. Purchase if available

**Cost**: ~$10-15 for first year

---

## Step 2: Configure DNS (10 minutes)

You'll set up 3 subdomains:

1. `api.ethhook.io` → admin-api service
2. `app.ethhook.io` → leptos-portal service
3. `ethhook.io` → main website (optional, redirect to app)

### For Cloudflare DNS

1. Go to Cloudflare dashboard
2. Click your domain
3. Go to "DNS" tab
4. Add these records (values from Railway):

#### Record 1: API Backend
```
Type: CNAME
Name: api
Target: <get from Railway - see below>
Proxy: Off (gray cloud)
TTL: Auto
```

#### Record 2: Frontend App
```
Type: CNAME
Name: app
Target: <get from Railway - see below>
Proxy: Off (gray cloud)
TTL: Auto
```

#### Record 3: Root Domain (Optional)
```
Type: CNAME
Name: @
Target: app.ethhook.io
Proxy: On (orange cloud)
TTL: Auto
```

### Where to Get Railway Target Values

1. Go to Railway dashboard
2. Click on service (e.g., admin-api)
3. Click "Settings" tab
4. Scroll to "Domains" section
5. Click "Custom Domain"
6. Enter your domain: `api.ethhook.io`
7. Railway will show you the CNAME target

**Example**:
```
CNAME: api.ethhook.io → admin-api.up.railway.app
```

Copy the right side value to your DNS.

### For Namecheap DNS

1. Go to Namecheap dashboard
2. Click "Manage" next to your domain
3. Go to "Advanced DNS" tab
4. Add CNAME records:

```
Type: CNAME Record
Host: api
Value: <railway-provided-value>
TTL: Automatic

Type: CNAME Record
Host: app
Value: <railway-provided-value>
TTL: Automatic
```

---

## Step 3: Add Custom Domains to Railway (5 minutes)

### For admin-api Service:

1. Railway Dashboard → admin-api service
2. Settings → Domains → Custom Domain
3. Enter: `api.ethhook.io`
4. Click "Add Domain"
5. Railway will show DNS instructions (you already did this!)
6. Wait 5-10 minutes for DNS propagation

### For leptos-portal Service:

1. Railway Dashboard → leptos-portal service
2. Settings → Domains → Custom Domain
3. Enter: `app.ethhook.io`
4. Click "Add Domain"
5. Wait 5-10 minutes

### Verify DNS Propagation

```bash
# Check if DNS is propagated
dig api.ethhook.io
dig app.ethhook.io

# Should show CNAME pointing to Railway

# Or use online tool
https://dnschecker.org/
```

---

## Step 4: SSL Certificates (Automatic)

Railway automatically provisions SSL certificates via Let's Encrypt.

**Timeline**:
- DNS propagation: 5-10 minutes
- SSL certificate: 5-10 minutes after DNS
- Total: ~15-20 minutes

**How to check**:
1. Go to Railway → Service → Settings → Domains
2. Look for green checkmark next to custom domain
3. Status changes from "Pending" → "Active"

**Test**:
```bash
curl https://api.ethhook.io/api/v1/health
# Should return: {"status":"healthy",...}

# Check SSL certificate
curl -vI https://api.ethhook.io/api/v1/health 2>&1 | grep -i 'SSL\|certificate'
```

---

## Step 5: Update Environment Variables (5 minutes)

After custom domains are active, update your environment variables:

### admin-api Service

Railway → admin-api → Variables → Edit:

```bash
CORS_ALLOWED_ORIGINS=https://app.ethhook.io
```

Redeploy admin-api after changing.

### leptos-portal Service (if needed)

If your frontend makes API calls, update the API URL:

```bash
VITE_API_URL=https://api.ethhook.io
```

---

## Step 6: Update Documentation (5 minutes)

Update references in your docs:

```bash
# Find all references to Railway URLs
grep -r "railway.app" docs/

# Update to custom domains
# Example:
# https://admin-api.up.railway.app → https://api.ethhook.io
# https://leptos-portal.up.railway.app → https://app.ethhook.io
```

Update in:
- README.md
- Deployment guides
- API documentation
- Frontend code (if hardcoded)

---

## Subdomain Strategy

### Recommended Setup:

```
ethhook.io              → Redirect to app.ethhook.io
app.ethhook.io          → Leptos Frontend (user-facing)
api.ethhook.io          → Admin API (backend)
docs.ethhook.io         → Documentation (optional)
status.ethhook.io       → Status page (optional)
blog.ethhook.io         → Blog (optional, future)
```

### Minimal Setup (MVP):

```
ethhook.io              → Frontend
api.ethhook.io          → Backend API
```

---

## Custom Domain Checklist

### Before You Start:
- [ ] Domain purchased
- [ ] Access to DNS management (Cloudflare/Namecheap)
- [ ] Railway services deployed and running

### DNS Configuration:
- [ ] CNAME record for `api` created
- [ ] CNAME record for `app` created
- [ ] DNS propagation verified (10-15 minutes)

### Railway Configuration:
- [ ] Custom domain added to admin-api
- [ ] Custom domain added to leptos-portal
- [ ] SSL certificates active (green checkmark)
- [ ] Health checks passing on new URLs

### Application Updates:
- [ ] CORS updated in admin-api
- [ ] API URL updated in frontend (if needed)
- [ ] Documentation updated with new URLs
- [ ] Old URLs redirecting or deprecated

### Testing:
- [ ] `https://api.ethhook.io/api/v1/health` works
- [ ] `https://app.ethhook.io` loads frontend
- [ ] SSL certificate valid (no browser warnings)
- [ ] CORS working (no console errors)
- [ ] API calls from frontend working

---

## Troubleshooting

### Issue: DNS Not Propagating

**Symptoms**: Domain doesn't resolve after 30 minutes

**Solutions**:
```bash
# Check DNS from multiple locations
https://dnschecker.org/

# Clear local DNS cache (Mac)
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder

# Try different DNS server
dig @8.8.8.8 api.ethhook.io

# Check TTL (should be low during setup)
dig api.ethhook.io | grep TTL
```

### Issue: SSL Certificate Not Provisioned

**Symptoms**: Railway shows "Pending" for SSL

**Solutions**:
1. Wait longer (can take up to 30 minutes)
2. Verify DNS is pointing correctly
3. Remove and re-add custom domain in Railway
4. Check Railway status page: https://status.railway.app/

### Issue: CORS Errors After Domain Change

**Symptoms**: Frontend shows CORS errors in console

**Solutions**:
1. Verify `CORS_ALLOWED_ORIGINS` in admin-api includes new domain
2. Ensure using `https://` not `http://`
3. No trailing slash in origin URL
4. Redeploy admin-api after changing env vars

```bash
# Check CORS headers
curl -H "Origin: https://app.ethhook.io" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     https://api.ethhook.io/api/v1/health -i

# Should see:
# Access-Control-Allow-Origin: https://app.ethhook.io
```

### Issue: Old Railway URL Still Works

**This is normal!** Railway doesn't remove the old URL.

**Options**:
1. Leave both URLs active (old continues to work)
2. Add redirect in code to force custom domain
3. Communicate new URL to users

---

## Cost Summary

### One-Time Costs:
- Domain registration: $10-15

### Annual Costs:
- Domain renewal: $10-15/year

### Railway Costs (No Change):
- Railway pricing same whether using custom domain or not
- SSL certificates: Free (included)

**Total Additional Cost**: ~$1/month ($12/year)

---

## Advanced: Multiple Environments

For production vs staging:

### Production:
```
ethhook.io              → Production frontend
api.ethhook.io          → Production API
```

### Staging:
```
staging.ethhook.io      → Staging frontend
api-staging.ethhook.io  → Staging API
```

### Development:
```
dev.ethhook.io          → Dev frontend
api-dev.ethhook.io      → Dev API
```

**Setup**: Create separate Railway projects for each environment, point different subdomains.

---

## Security: HTTPS Enforcement

Railway automatically enforces HTTPS, but you can double-check:

```bash
# Try HTTP (should redirect to HTTPS)
curl -I http://api.ethhook.io/api/v1/health

# Should see:
# HTTP/1.1 301 Moved Permanently
# Location: https://api.ethhook.io/api/v1/health
```

If not redirecting, Railway handles this automatically once SSL is active.

---

## Post-Setup Tasks

After custom domain is live:

1. **Update OAuth redirects** (if using OAuth):
   - Google OAuth: Add new redirect URL
   - GitHub OAuth: Add new callback URL

2. **Update API documentation**:
   - Swagger/OpenAPI base URL
   - Postman collection base URL

3. **Update marketing materials**:
   - Website
   - Social media
   - Business cards

4. **Set up monitoring**:
   - UptimeRobot: Monitor `https://api.ethhook.io/health`
   - StatusCake: Monitor uptime

5. **Configure analytics** (if using):
   - Google Analytics: Update property
   - Mixpanel: Update project

---

## Success Criteria

Your custom domain is ready when:

- ✅ `https://api.ethhook.io/api/v1/health` returns healthy
- ✅ `https://app.ethhook.io` loads your frontend
- ✅ SSL certificate valid (no browser warnings)
- ✅ CORS working (no console errors)
- ✅ Users can register and login
- ✅ API calls working from frontend
- ✅ All old Railway URLs still work (backwards compatible)

---

## Maintenance

### Monthly:
- [ ] Check SSL certificate expiry (Railway auto-renews)
- [ ] Verify DNS records still correct
- [ ] Test all custom domain endpoints

### Annually:
- [ ] Renew domain registration
- [ ] Review and update subdomain strategy
- [ ] Consider additional domains (country-specific, typo domains)

---

**Questions?** Railway docs: https://docs.railway.app/deploy/exposing-your-app#custom-domains

**Need help?** Railway Discord: https://discord.gg/railway (search for "custom domain")
