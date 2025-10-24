# Documentation Organization Guide

**Purpose**: Separate public and internal documentation to protect sensitive information

---

## Current Problem

Your repository has internal documents mixed with public ones:
- `LOGIN_CREDENTIALS.md` - Contains passwords! 🚨
- `MVP_ISSUES_AND_SOLUTIONS.md` - Internal planning
- `FIXED_ISSUES.md` - Internal status
- `PRIORITY_1_COMPLETE.md` - Internal tracking
- `SERVICES_STATUS.md` - Internal status
- `UI_DATA_ISSUES_AND_FIXES.md` - Internal bugs
- `E2E_TEST_STATUS.md` - Internal testing

**Risk**: These files contain sensitive information and should NOT be in git!

---

## Solution: Two-Tier Documentation

### Tier 1: Public Documentation (in git, visible to all)

**Location**: `docs/public/` or root level

**Contents**:
- README.md
- API documentation
- Deployment guides
- Architecture diagrams
- Contributing guidelines
- User guides

**Who can see**: Everyone (GitHub visitors, potential users, contributors)

###Tier 2: Internal Documentation (NOT in git)

**Location**: `docs/internal/` (added to .gitignore)

**Contents**:
- Credentials and passwords
- API keys and secrets
- Cost/billing information
- Internal issue tracking
- Sprint planning
- Private TODO lists
- Team-only notes

**Who can see**: Only you and your team (stored locally or private wiki)

---

## Step-by-Step Migration

### Step 1: Create Internal Docs Directory (2 minutes)

```bash
cd /Users/igor/rust_projects/capstone0
mkdir -p docs/internal
```

### Step 2: Move Sensitive Files (5 minutes)

```bash
# Move internal files to internal directory
mv LOGIN_CREDENTIALS.md docs/internal/
mv MVP_ISSUES_AND_SOLUTIONS.md docs/internal/
mv FIXED_ISSUES.md docs/internal/
mv PRIORITY_1_COMPLETE.md docs/internal/
mv SERVICES_STATUS.md docs/internal/
mv UI_DATA_ISSUES_AND_FIXES.md docs/internal/
mv UI_IS_LIVE.md docs/internal/
mv E2E_TEST_STATUS.md docs/internal/

# Keep frontend MVP doc but move to internal if it has sensitive info
mv docs/FRONTEND_MVP_COMPLETE.md docs/internal/
```

### Step 3: Update .gitignore (1 minute)

Add to your `.gitignore`:

```gitignore
# Internal documentation (DO NOT COMMIT)
docs/internal/
**/credentials*.md
**/LOGIN_CREDENTIALS*.md
```

**OR** replace your entire `.gitignore` with `.gitignore.production` (which I created):

```bash
cp .gitignore.production .gitignore
```

### Step 4: Remove from Git History (5 minutes)

**CRITICAL**: These files are already in git history!

```bash
# Remove sensitive files from git (keeps local copy)
git rm --cached LOGIN_CREDENTIALS.md
git rm --cached MVP_ISSUES_AND_SOLUTIONS.md
git rm --cached FIXED_ISSUES.md
git rm --cached PRIORITY_1_COMPLETE.md
git rm --cached SERVICES_STATUS.md
git rm --cached UI_DATA_ISSUES_AND_FIXES.md
git rm --cached UI_IS_LIVE.md
git rm --cached E2E_TEST_STATUS.md
git rm --cached docs/FRONTEND_MVP_COMPLETE.md

# Commit the removal
git commit -m "chore: Remove sensitive internal documentation from repository"

# Push
git push origin main
```

**Note**: Files will still exist in git history. To completely remove:

```bash
# WARNING: This rewrites git history!
# Only do this if repository is private or you haven't shared it yet

git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch LOGIN_CREDENTIALS.md" \
  --prune-empty --tag-name-filter cat -- --all

# Force push (WARNING: This affects all clones)
git push origin --force --all
```

### Step 5: Organize Public Docs (10 minutes)

Create public documentation structure:

```bash
mkdir -p docs/public

# Move public-safe docs to public directory
mv docs/SYSTEM_ARCHITECTURE.md docs/public/ARCHITECTURE.md
mv docs/RAILWAY_DEPLOYMENT_GUIDE.md docs/public/DEPLOYMENT.md
mv docs/PRODUCTION_READINESS_CHECKLIST.md docs/public/DEPLOYMENT_CHECKLIST.md

# Create public README
cat > docs/public/README.md << 'EOF'
# EthHook Documentation

## Public Documentation

- [Architecture](ARCHITECTURE.md) - System architecture and design
- [Deployment Guide](DEPLOYMENT.md) - How to deploy to Railway
- [Deployment Checklist](DEPLOYMENT_CHECKLIST.md) - Pre-deployment checklist
- [API Reference](API.md) - REST API documentation (coming soon)

## Getting Started

1. Read the [Architecture](ARCHITECTURE.md) to understand the system
2. Follow the [Deployment Guide](DEPLOYMENT.md) to deploy
3. Use the [API Reference](API.md) to integrate

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root directory.
EOF
```

---

## Recommended Directory Structure

```
ethhook/
├── README.md                          # Public: Project overview
├── CONTRIBUTING.md                    # Public: How to contribute
├── LICENSE                            # Public: MIT license
├── DEPLOYMENT_QUICKSTART.md           # Public: Quick deploy guide
│
├── docs/
│   ├── public/                        # PUBLIC DOCS (in git)
│   │   ├── README.md
│   │   ├── ARCHITECTURE.md
│   │   ├── DEPLOYMENT.md
│   │   ├── API.md
│   │   └── CONTRIBUTING.md
│   │
│   └── internal/                      # INTERNAL DOCS (NOT in git)
│       ├── credentials.md             # Passwords, API keys
│       ├── login_credentials.md       # User/pass for services
│       ├── mvp_issues.md              # Internal issues tracking
│       ├── fixed_issues.md            # Completed issues
│       ├── services_status.md         # Current status
│       ├── costs.md                   # Billing information
│       └── planning/                  # Sprint planning, TODOs
│
├── crates/                            # Source code
└── scripts/                           # Deployment scripts
```

---

## What to Keep Public vs Internal

### ✅ Safe for Public (Keep in Git)

- **Architecture diagrams** - Shows system design (no secrets)
- **API documentation** - Helps users integrate
- **Deployment guides** - Helps others deploy their own instance
- **Code** - Your actual Rust source code
- **Tests** - Unit and integration tests
- **Contributing guidelines** - How others can contribute
- **README** - Project description

### ❌ Keep Internal (NOT in Git)

- **Credentials** - Passwords, API keys, JWT secrets
- **Cost information** - Your AWS/Railway bills
- **Internal planning** - Sprint planning, TODOs
- **Issue tracking** - Your internal Jira/GitHub issues
- **Customer data** - User emails, usage stats
- **Private notes** - Team discussions, decisions
- **Staging/production URLs** - Your actual deployed URLs
- **Database connection strings** - With real passwords

---

## Alternative: Private Wiki

Instead of `docs/internal/`, you can use:

### Option 1: GitHub Wiki (Private)

1. Go to your repo settings
2. Enable wiki (make it private)
3. Store internal docs there

**Pros**: Built into GitHub, version controlled
**Cons**: Separate from main repo

### Option 2: Notion/Confluence

1. Create private workspace
2. Move internal docs there
3. Share with team only

**Pros**: Better formatting, collaboration
**Cons**: Another tool to maintain

### Option 3: Encrypted Git Repo

1. Create separate private repo: `ethhook-internal`
2. Store all sensitive docs there
3. Only team has access

**Pros**: Still version controlled, secure
**Cons**: Need to sync between repos

---

## Security Best Practices

### 1. Never Commit Secrets

```bash
# ❌ BAD
git commit -m "Added API key: sk_live_123456..."

# ✅ GOOD
# Add to .env (which is in .gitignore)
# Reference in code via environment variable
```

### 2. Use Environment Variables

```rust
// ❌ BAD
let api_key = "sk_live_123456";

// ✅ GOOD
let api_key = std::env::var("API_KEY")?;
```

### 3. Scan for Secrets

```bash
# Install gitleaks
brew install gitleaks

# Scan repository
gitleaks detect --source . --verbose

# Scan before commit
gitleaks protect --staged
```

### 4. Use .env.example

```bash
# .env.example (committed to git)
API_KEY=your_api_key_here
DATABASE_URL=postgresql://user:password@localhost/db

# .env (NOT committed to git)
API_KEY=sk_live_actual_key_123456
DATABASE_URL=postgresql://real_user:real_password@prod.db/ethhook
```

---

## Verification Checklist

After organizing docs:

- [ ] All sensitive files moved to `docs/internal/`
- [ ] `docs/internal/` added to `.gitignore`
- [ ] Sensitive files removed from git with `git rm --cached`
- [ ] Committed and pushed changes
- [ ] Verified files not in GitHub web interface
- [ ] Public docs remain accessible in `docs/public/`
- [ ] README updated with new structure
- [ ] Team notified of new documentation location

---

## Emergency: Leaked Credentials

If you accidentally committed credentials:

### 1. Change the Credentials Immediately

- Regenerate API keys
- Change passwords
- Rotate JWT secrets
- Update .env files

### 2. Remove from Git History

```bash
# Use BFG Repo Cleaner (easier than git filter-branch)
brew install bfg

# Remove file completely
bfg --delete-files LOGIN_CREDENTIALS.md

# Clean up
git reflog expire --expire=now --all
git gc --prune=now --aggressive

# Force push
git push origin --force --all
```

### 3. Notify Team

- Inform all team members
- Update deployed services with new credentials
- Monitor for unauthorized access

---

## Summary

**DO THIS NOW** (15 minutes):

1. Create `docs/internal/` directory
2. Move sensitive files there
3. Update `.gitignore`
4. Remove from git: `git rm --cached <files>`
5. Commit and push

**Result**:
- ✅ Sensitive information protected
- ✅ Public docs remain accessible
- ✅ Clear separation of concerns
- ✅ Production-ready repository

---

**Need help?** Ask me to execute any of these steps for you!
