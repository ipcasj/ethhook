# Production Deployment - Issues & Fixes Plan

**Date**: October 21, 2025
**Status**: Action Required Before Deployment

---

## Issues Identified

### 1. Frontend Hosting ⚠️
**Issue**: Suggested Vercel, but Railway can host everything
**Impact**: Extra service dependency, split deployment

### 2. Codebase State ⚠️
**Issue**: Many uncommitted changes in git
**Impact**: Unclear what's production-ready vs work-in-progress

### 3. Code Quality ⚠️
**Issue**: No formal Rust code audit performed
**Impact**: Potential non-idiomatic code, performance issues

### 4. Documentation Security 🔒
**Issue**: Internal/sensitive docs mixed with public docs
**Impact**: Risk of exposing internal information

### 5. Custom Domain 🌐
**Issue**: Railway default URLs are not branded
**Impact**: Unprofessional URLs for production

---

## Action Plan

### Action 1: Frontend on Railway (30 minutes)

**Task**: Create proper Dockerfile for Leptos frontend on Railway

**Steps**:
1. Create `crates/leptos-portal/Dockerfile`
2. Create `crates/leptos-portal/nginx.conf`
3. Test local build
4. Deploy to Railway

**Files to create**:
- `crates/leptos-portal/Dockerfile`
- `crates/leptos-portal/nginx.conf`
- Update deployment guides

---

### Action 2: Git Cleanup (15 minutes)

**Task**: Review and commit all changes

**Steps**:
1. Review uncommitted changes
2. Commit production-ready changes
3. Create `.gitignore` entries for sensitive files
4. Tag release: `v0.1.0-mvp`

**Commands**:
```bash
git status
git add <production-ready-files>
git commit -m "feat: Production deployment preparation"
git tag v0.1.0-mvp
git push origin main --tags
```

---

### Action 3: Rust Code Audit (60 minutes)

**Task**: Review Rust code for best practices

**Areas to check**:
1. Error handling patterns
2. Async/await usage
3. Resource cleanup
4. Memory safety
5. Performance bottlenecks
6. Security issues

**Tools**:
```bash
cargo clippy --all-targets --all-features
cargo fmt --check
cargo audit
```

**Create**: `RUST_CODE_AUDIT_REPORT.md`

---

### Action 4: Documentation Security (30 minutes)

**Task**: Separate public and internal documentation

**Structure**:
```
docs/
├── public/              # Goes to GitHub, visible to all
│   ├── README.md
│   ├── API.md
│   ├── DEPLOYMENT.md
│   └── ARCHITECTURE.md
└── internal/            # Added to .gitignore, NOT in repo
    ├── credentials.md
    ├── costs.md
    ├── issues.md
    └── planning.md
```

**Files to move to internal**:
- `MVP_ISSUES_AND_SOLUTIONS.md`
- `FIXED_ISSUES.md`
- `LOGIN_CREDENTIALS.md`
- `PRIORITY_1_COMPLETE.md`
- `SERVICES_STATUS.md`
- `UI_DATA_ISSUES_AND_FIXES.md`
- Any files with passwords, keys, or internal planning

**Create**: Updated `.gitignore`

---

### Action 5: Custom Domain (20 minutes)

**Task**: Configure custom domain on Railway

**Options**:
1. Buy domain (Namecheap, Google Domains, Cloudflare)
2. Configure DNS
3. Add to Railway

**Steps**:
1. Purchase domain: `ethhook.com` or `ethhook.io`
2. Railway → Service → Settings → Domains
3. Add custom domain
4. Update DNS records (Railway provides instructions)
5. Wait for SSL certificate (automatic, 5-10 minutes)

**Cost**: $10-15/year for domain

---

## Detailed Fixes

### Fix 1: Frontend Dockerfile

Create: `crates/leptos-portal/Dockerfile`

```dockerfile
# Multi-stage build for Leptos WASM frontend

# Stage 1: Build WASM
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install trunk and wasm target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Copy source code
COPY Cargo.toml ./
COPY index.html ./
COPY style.css ./
COPY src ./src

# Build frontend
RUN trunk build --release

# Stage 2: Serve with Nginx
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD wget --quiet --tries=1 --spider http://localhost/ || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
```

Create: `crates/leptos-portal/nginx.conf`

```nginx
events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Performance
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_types text/css application/javascript application/json application/wasm;

    server {
        listen 80;
        server_name _;

        root /usr/share/nginx/html;
        index index.html;

        # SPA routing - all routes go to index.html
        location / {
            try_files $uri $uri/ /index.html;
        }

        # Cache static assets
        location ~* \.(js|css|wasm|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # Health check endpoint
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }

        # Security headers
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
    }
}
```

---

### Fix 2: Secure .gitignore

Update: `.gitignore`

```gitignore
# Existing entries...

# Internal documentation (DO NOT COMMIT)
docs/internal/
**/credentials*.md
**/LOGIN_CREDENTIALS*.md
**/FIXED_ISSUES*.md
**/MVP_ISSUES*.md
**/PRIORITY_*.md
**/SERVICES_STATUS*.md
**/UI_DATA_ISSUES*.md
**/*_INTERNAL*.md

# Environment files with real credentials
.env.production
.env.local

# Cost/billing information
**/costs*.md
**/billing*.md

# Local development files
*.log
*.swp
*.swo
*~

# IDE files
.vscode/settings.json
.idea/workspace.xml
```

---

### Fix 3: Public Documentation Structure

Create: `docs/public/README.md`

```markdown
# EthHook Public Documentation

This directory contains documentation safe for public viewing.

## Contents

- **[API.md](API.md)** - API reference
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Deployment guide
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines

## Private Documentation

Internal documentation is not included in this repository for security reasons.
```

Move these files to `docs/public/`:
- `SYSTEM_ARCHITECTURE.md` → `docs/public/ARCHITECTURE.md`
- `RAILWAY_DEPLOYMENT_GUIDE.md` → `docs/public/DEPLOYMENT.md`

---

### Fix 4: Custom Domain Configuration

Create: `docs/public/CUSTOM_DOMAIN_SETUP.md`

```markdown
# Custom Domain Setup on Railway

## Step 1: Purchase Domain

Recommended registrars:
- **Cloudflare**: $8-10/year, includes DDoS protection
- **Namecheap**: $10-12/year
- **Google Domains**: $12/year

Domain suggestions:
- `ethhook.io`
- `ethhook.dev`
- `webhooks.eth` (not real DNS, use .io/.dev)

## Step 2: Configure DNS

### For API (Backend)

1. Railway → admin-api → Settings → Domains
2. Click "Custom Domain"
3. Enter: `api.yourdomain.com`
4. Railway shows DNS records needed

Add to your DNS:
```
Type: CNAME
Name: api
Value: <railway-provided-value>
```

### For Frontend

1. Railway → leptos-portal → Settings → Domains
2. Enter: `app.yourdomain.com` or `yourdomain.com`
3. Add DNS records as shown

## Step 3: SSL Certificate

Railway automatically provisions SSL certificates via Let's Encrypt.

**Time**: 5-10 minutes after DNS propagation

## Step 4: Update Environment Variables

Update in Railway:

```bash
# admin-api
CORS_ALLOWED_ORIGINS=https://app.yourdomain.com

# Frontend (if needed)
VITE_API_URL=https://api.yourdomain.com
```

## Step 5: Verify

```bash
curl https://api.yourdomain.com/api/v1/health
# Should return: {"status":"healthy"}
```

## Costs

- Domain: $10-15/year
- SSL: Free (Let's Encrypt via Railway)
- Total: ~$1/month
```

---

## Priority Order

### CRITICAL (Do before deployment)

1. ✅ **Fix 2: Secure .gitignore** (10 min)
   - Prevent committing sensitive files

2. ✅ **Action 4: Move internal docs** (20 min)
   - Protect internal information

3. ✅ **Action 2: Git cleanup** (15 min)
   - Clean state for deployment

### HIGH (Do during deployment)

4. ✅ **Fix 1: Frontend Dockerfile** (30 min)
   - Self-contained deployment

5. ✅ **Action 5: Custom domain** (20 min)
   - Professional URLs

### MEDIUM (Do after deployment)

6. ⚠️ **Action 3: Code audit** (60 min)
   - Code quality improvements

---

## Timeline

**Today (Before Deployment)**:
- 45 minutes for CRITICAL fixes
- Commit changes
- Ready to deploy

**During Deployment**:
- 50 minutes for HIGH priority items
- Deploy with proper frontend
- Configure custom domain

**Week 1 (After Deployment)**:
- 60 minutes for code audit
- Implement improvements
- Release v0.1.1

---

## Checklist

### Before Deployment

- [ ] Update .gitignore with sensitive patterns
- [ ] Move internal docs to docs/internal/
- [ ] Review git status
- [ ] Commit production-ready changes
- [ ] Create git tag: v0.1.0-mvp
- [ ] Push to GitHub

### During Deployment

- [ ] Create frontend Dockerfile
- [ ] Create nginx.conf
- [ ] Test frontend build locally
- [ ] Deploy frontend to Railway
- [ ] Purchase custom domain (optional)
- [ ] Configure DNS for custom domain
- [ ] Wait for SSL certificate
- [ ] Update CORS settings

### After Deployment

- [ ] Run cargo clippy
- [ ] Run cargo audit
- [ ] Review code for best practices
- [ ] Document improvements needed
- [ ] Create issues for follow-up work

---

## Help Needed?

Each fix has detailed instructions. If you need help with any:

1. Check the fix instructions above
2. Railway docs: https://docs.railway.app/
3. Ask me for specific guidance

---

**Status**: Plan created, waiting for your approval to proceed
