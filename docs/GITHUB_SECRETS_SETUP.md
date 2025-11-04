# GitHub Secrets Configuration Guide

This guide explains how to configure GitHub repository secrets for the CI/CD pipeline to enable automatic deployment to your DigitalOcean production server.

## 🔐 Required Secrets

To enable production deployment, you need to configure the following secrets in your GitHub repository:

### Navigate to Secrets Settings

1. Go to your GitHub repository: `https://github.com/ipcasj/ethhook`
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** for each secret below

---

## 📋 Secrets to Configure

### 1. `DROPLET_HOST`

**Description:** IP address of your DigitalOcean droplet  
**Value:** `104.248.15.178`  
**Usage:** SSH connection target for deployment

### 2. `DROPLET_USER`

**Description:** SSH username for droplet access  
**Value:** `root`  
**Usage:** SSH username for connecting to the droplet

### 3. `DROPLET_SSH_KEY`

**Description:** Private SSH key for authentication  
**How to get:** Copy your private SSH key that has access to the droplet

#### Option A: Using existing SSH key

```bash
# Display your private key
cat ~/.ssh/id_rsa
# OR if you use a different key
cat ~/.ssh/id_ed25519
```

#### Option B: Create a new deploy-specific key (recommended)

```bash
# Generate a new SSH key specifically for GitHub Actions
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_deploy_key

# Copy the private key (this goes in DROPLET_SSH_KEY secret)
cat ~/.ssh/github_deploy_key

# Copy the public key to your droplet
ssh-copy-id -i ~/.ssh/github_deploy_key.pub your_user@104.248.15.178
```

**⚠️ Important:** Copy the **entire** private key including:

```text
-----BEGIN OPENSSH PRIVATE KEY-----
...all the key content...
-----END OPENSSH PRIVATE KEY-----
```

### 4. `PRODUCTION_URL`

**Description:** Base URL for health checks and monitoring  
**Value:** `http://104.248.15.178:3000`  
**Usage:** Health check endpoint validation after deployment (backend API)
**Note:** The UI runs on port 3002, but health checks use the API on port 3000

### 5. `CODECOV_TOKEN` (Optional)

**Description:** Token for uploading code coverage reports  
**Value:** Get from https://codecov.io after setting up your repository  
**Usage:** Coverage reporting in CI (optional, doesn't block deployment)

---

## 🎯 Quick Setup Commands

Run these commands on your DigitalOcean droplet to prepare for automated deployment:

```bash
# 1. Ensure git repository is set up
cd ~/ethhook || cd ~/rust_projects/capstone0
git remote -v
# Should show: origin  https://github.com/ipcasj/ethhook.git

# 2. Create production docker-compose file if not exists
ls docker-compose.prod.yml
# If missing, copy from docker-compose.yml and adjust for production

# 3. Verify Docker is installed
docker --version
docker compose version

# 4. Test SSH access from your local machine
ssh your_user@104.248.15.178 "cd ~/ethhook && git status"
```

---

## ✅ Verification Steps

After configuring all secrets in GitHub:

### 1. Test Manual Deployment

1. Go to **Actions** tab in GitHub
2. Select **Production Deployment** workflow
3. Click **Run workflow** → **Run workflow** button
4. Watch the deployment logs

### 2. Verify Automatic Deployment

```bash
# Make a small change and push to main
echo "# Testing CI/CD" >> README.md
git add README.md
git commit -m "test: Verify CI/CD deployment"
git push origin main

# Watch GitHub Actions automatically deploy
```

### 3. Check Deployment Success

- GitHub Actions should show ✅ green checkmark
- Visit UI: `http://104.248.15.178:3002` (Next.js frontend)
- Check API: `curl http://104.248.15.178:3000/api/v1/health`

---

## 🔧 Troubleshooting

### SSH Connection Fails

```bash
# Test SSH connection manually
ssh -i ~/.ssh/your_key your_user@104.248.15.178

# Check SSH key permissions (must be 600)
chmod 600 ~/.ssh/your_key

# Verify public key is in authorized_keys on droplet
ssh 104.248.15.178 "cat ~/.ssh/authorized_keys"
```

### Docker Compose Not Found

```bash
# Install Docker Compose V2 on droplet
ssh 104.248.15.178 << 'EOF'
  sudo apt-get update
  sudo apt-get install -y docker-compose-plugin
  docker compose version
EOF
```

### Health Check Fails

```bash
# Check if services are running on droplet
ssh 104.248.15.178 "docker ps"

# Check service logs
ssh 104.248.15.178 "docker logs admin-api"

# Manually test health endpoint
curl http://104.248.15.178:3000/api/v1/health
```

### Git Pull Fails

```bash
# Ensure GitHub has SSH key access (if using SSH URLs)
# OR ensure HTTPS cloning works without credentials

# Switch to HTTPS if needed
ssh 104.248.15.178 << 'EOF'
  cd ~/ethhook
  git remote set-url origin https://github.com/ipcasj/ethhook.git
EOF
```

---

## 🚀 Deployment Workflow

Once secrets are configured, every push to `main` will:

1. ✅ **Run CI tests** (lint, test, build)
2. 🔐 **SSH into droplet**
3. 📥 **Pull latest code** from GitHub
4. 🛑 **Stop services** (docker-compose down)
5. 🔨 **Build images** (docker-compose build)
6. 🗃️ **Run migrations** (sqlx migrate run)
7. ▶️ **Start services** (docker-compose up -d)
8. 🏥 **Health check** (verify API responds)
9. 📊 **Report status** (GitHub Actions summary)

---

## 🎉 Post-Setup

After successful setup:

- **Monitor deployments:** GitHub Actions tab shows deployment status
- **View logs:** SSH into droplet and run `docker logs <service>`
- **Rollback:** Use workflow dispatch with rollback option if needed
- **Zero-downtime:** Consider blue-green deployment for production traffic

---

## 📚 Additional Resources

- [GitHub Actions Secrets Documentation](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [DigitalOcean SSH Keys Setup](https://docs.digitalocean.com/products/droplets/how-to/add-ssh-keys/)
- [Docker Compose Production Best Practices](https://docs.docker.com/compose/production/)

---

## 🔒 Security Best Practices

1. **Use deploy-specific SSH keys** (not your personal key)
2. **Rotate keys periodically** (every 90 days)
3. **Limit key permissions** (read-only where possible)
4. **Enable GitHub branch protection** (require PR reviews)
5. **Use environment-specific secrets** (separate staging/production)
6. **Monitor deployment logs** (set up alerts for failures)
7. **Keep secrets secure** (never commit to repository)

---

## 💡 Next Steps

1. ✅ Configure all GitHub secrets (15 minutes)
2. ✅ Test manual deployment workflow (5 minutes)
3. ✅ Make test commit to trigger auto-deployment (2 minutes)
4. ✅ Set up deployment notifications (Slack/Discord/Email)
5. ✅ Create staging environment (recommended)
6. ✅ Configure custom domain with SSL (see CUSTOM_DOMAIN_SETUP.md)
7. ✅ Set up monitoring and alerts (see GRAFANA_DASHBOARD_SETUP.md)
