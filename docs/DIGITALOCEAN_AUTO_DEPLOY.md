# DigitalOcean Automated Deployment Setup

This guide explains how to set up automated deployment to your DigitalOcean droplet.

## Overview

The deployment workflow (`.github/workflows/deploy-digitalocean.yml`) automatically deploys to production when:
- ✅ CI workflow completes successfully on `main` branch
- ✅ Manual trigger from GitHub Actions UI

## Required GitHub Secrets

You need to configure these secrets in your GitHub repository:

### 1. `DO_SSH_PRIVATE_KEY`
Your SSH private key for accessing the DigitalOcean droplet.

**To generate and add:**
```bash
# On your local machine, generate a new SSH key pair (if you don't have one)
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_deploy

# Copy the public key to your DigitalOcean droplet
ssh-copy-id -i ~/.ssh/github_deploy.pub root@104.248.15.178

# Display the private key (copy this to GitHub secret)
cat ~/.ssh/github_deploy
```

**Add to GitHub:**
1. Go to: `https://github.com/ipcasj/ethhook/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `DO_SSH_PRIVATE_KEY`
4. Value: Paste the entire private key (including `-----BEGIN OPENSSH PRIVATE KEY-----` and `-----END OPENSSH PRIVATE KEY-----`)

### 2. `DO_HOST`
Your DigitalOcean droplet IP address.

**Value:** `104.248.15.178`

**Add to GitHub:**
1. Go to: `https://github.com/ipcasj/ethhook/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `DO_HOST`
4. Value: `104.248.15.178`

## Deployment Workflow

### Automatic Deployment
1. Push code to `main` branch
2. CI workflow runs (builds, tests, pushes Docker images)
3. If CI passes, deployment workflow automatically triggers
4. Services are updated on DigitalOcean with zero-downtime deployment

### Manual Deployment
1. Go to: `https://github.com/ipcasj/ethhook/actions/workflows/deploy-digitalocean.yml`
2. Click "Run workflow"
3. Select `main` branch
4. Click "Run workflow"

## What Gets Deployed

The workflow deploys these services:
- ✅ **pipeline** - WebSocket + batch processing + delivery
- ✅ **admin-api** - REST API
- ✅ **ui** - Next.js frontend
- ✅ **demo-webhook-receiver** - Demo webhook receiver

**Note:** ClickHouse is not redeployed (data persistence).

## Deployment Process

1. **Pull Images** - Downloads latest Docker images from GitHub Container Registry
2. **Stop Services** - Gracefully stops running containers
3. **Remove Containers** - Removes old containers
4. **Start Services** - Starts updated containers
5. **Health Check** - Verifies API is responding
6. **Cleanup** - Removes old Docker images to save disk space

## Server Prerequisites

Ensure your DigitalOcean droplet has:

```bash
# 1. Docker and Docker Compose installed
docker --version
docker compose version

# 2. Project directory exists
mkdir -p ~/ethhook

# 3. docker-compose.prod.yml is present
ls -l ~/ethhook/docker-compose.prod.yml

# 4. Environment variables are set
cat ~/ethhook/.env
```

## Monitoring Deployments

### View Workflow Runs
https://github.com/ipcasj/ethhook/actions/workflows/deploy-digitalocean.yml

### Check Deployment Logs
```bash
# SSH into your droplet
ssh root@104.248.15.178

# View running services
cd ~/ethhook
docker compose -f docker-compose.prod.yml ps

# View logs
docker compose -f docker-compose.prod.yml logs --tail=100 -f pipeline
docker compose -f docker-compose.prod.yml logs --tail=100 -f admin-api
docker compose -f docker-compose.prod.yml logs --tail=100 -f ui
```

## Rollback Procedure

If a deployment fails, you can rollback:

```bash
# SSH into droplet
ssh root@104.248.15.178
cd ~/ethhook

# Pull specific version (replace with commit SHA)
docker compose -f docker-compose.prod.yml pull
docker tag ghcr.io/ipcasj/ethhook-pipeline:latest ghcr.io/ipcasj/ethhook-pipeline:backup
docker pull ghcr.io/ipcasj/ethhook-pipeline:COMMIT_SHA
docker tag ghcr.io/ipcasj/ethhook-pipeline:COMMIT_SHA ghcr.io/ipcasj/ethhook-pipeline:latest

# Restart services
docker compose -f docker-compose.prod.yml up -d pipeline admin-api ui
```

## Troubleshooting

### Deployment Failed - SSH Connection
```
Error: Permission denied (publickey)
```
**Solution:** Verify `DO_SSH_PRIVATE_KEY` secret is correct and SSH key is added to droplet.

### Deployment Failed - Docker Login
```
Error: unauthorized: authentication required
```
**Solution:** Ensure GitHub Actions has access to GitHub Container Registry. Check package permissions.

### Services Not Starting
```bash
# SSH into droplet and check logs
ssh root@104.248.15.178
cd ~/ethhook
docker compose -f docker-compose.prod.yml logs --tail=200
```

### Disk Space Issues
```bash
# Clean up Docker resources
docker system prune -a -f
docker volume prune -f
```

## Security Best Practices

1. ✅ Use dedicated SSH key for deployments (not your personal key)
2. ✅ Rotate SSH keys periodically
3. ✅ Limit SSH key permissions (read-only access where possible)
4. ✅ Monitor GitHub Actions logs for suspicious activity
5. ✅ Use GitHub's secret scanning to detect leaked credentials

## Quick Setup Checklist

- [ ] Generate SSH key pair for GitHub Actions
- [ ] Add public key to DigitalOcean droplet (`~/.ssh/authorized_keys`)
- [ ] Add `DO_SSH_PRIVATE_KEY` secret to GitHub
- [ ] Add `DO_HOST` secret to GitHub
- [ ] Verify `docker-compose.prod.yml` exists on droplet
- [ ] Verify environment variables in `~/ethhook/.env`
- [ ] Test manual deployment workflow
- [ ] Verify automatic deployment on push to main

## Next Steps

After setup:
1. Push a change to `main` branch
2. Watch CI workflow complete
3. Watch deployment workflow automatically trigger
4. Verify services are updated on DigitalOcean
5. Check application is working: http://104.248.15.178:3001/health

---

**Need help?** Check GitHub Actions logs or SSH into the droplet to investigate issues.
