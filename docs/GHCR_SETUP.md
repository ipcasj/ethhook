# GitHub Container Registry Setup

## Problem
The Docker images are successfully built and pushed to GitHub Container Registry (ghcr.io), but the deployment server gets "denied" errors when trying to pull them, even though the packages are set to public.

## Root Cause
GitHub Container Registry requires authentication to pull images, even for public packages in many cases. This is a security feature of GHCR.

## Solution: Authenticate the Deployment Server

### Option 1: Using Personal Access Token (Recommended for initial setup)

1. **Create a GitHub Personal Access Token**:
   - Go to https://github.com/settings/tokens/new
   - Token name: `ethhook-deployment-readonly`
   - Expiration: Choose appropriate duration (90 days recommended)
   - Select scope: **`read:packages`** only
   - Click "Generate token"
   - **Copy the token immediately** (you won't see it again)

2. **Authenticate on the deployment server**:
   ```bash
   # SSH into the server
   ssh root@104.248.15.178

   # Login to GitHub Container Registry
   echo "YOUR_TOKEN_HERE" | docker login ghcr.io -u ipcasj --password-stdin
   ```

3. **Test pulling an image**:
   ```bash
   docker pull ghcr.io/ipcasj/ethhook-event-ingestor:latest
   ```

4. **Deploy with registry images**:
   ```bash
   cd ~/ethhook
   docker compose -f docker-compose.prod.yml pull
   docker compose -f docker-compose.prod.yml up -d
   ```

### Option 2: Using GitHub Actions Deploy Key (Recommended for production)

For automated deployments, use the existing `GITHUB_TOKEN` secret in the deployment workflow:

1. The `.github/workflows/digitalocean-deploy.yml` workflow already has access to `GITHUB_TOKEN`
2. Add a login step before deploying:
   ```yaml
   - name: Login to GitHub Container Registry
     run: |
       ssh ${{ secrets.DROPLET_USER }}@${{ secrets.DROPLET_HOST }} \
         "echo ${{ secrets.GITHUB_TOKEN }} | docker login ghcr.io -u ${{ github.actor }} --password-stdin"
   ```

## Verification

After authentication, you should be able to:

```bash
# List available tags
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://ghcr.io/v2/ipcasj/ethhook-event-ingestor/tags/list

# Pull any image
docker pull ghcr.io/ipcasj/ethhook-event-ingestor:latest
docker pull ghcr.io/ipcasj/ethhook-message-processor:latest
docker pull ghcr.io/ipcasj/ethhook-webhook-delivery:latest
docker pull ghcr.io/ipcasj/ethhook-ui:latest
docker pull ghcr.io/ipcasj/ethhook-demo-receiver:latest
```

## Current Status

✅ **Successfully built and pushed (5/6 images)**:
- event-ingestor
- message-processor
- webhook-delivery
- ui
- demo-receiver

❌ **Failed to build**:
- admin-api (needs investigation - separate issue)

## Benefits After Setup

Once authenticated:
- ✅ Fast deployments (pull pre-built images instead of building on droplet)
- ✅ No droplet crashes from memory-intensive Rust compilation
- ✅ Consistent images across environments
- ✅ Faster rollbacks (just change image tag)
- ✅ Reduced disk space usage on droplet

## Troubleshooting

### "denied: denied" error
- Means authentication is required or token lacks permissions
- Solution: Follow authentication steps above

### "manifest unknown" error
- Means the specific image tag doesn't exist
- Solution: Check available tags or trigger a new CI build

### Token expired
- PATs can expire based on the duration you set
- Solution: Create a new token and re-authenticate
