# Docker Registry Implementation

## Problem
The DigitalOcean droplet was becoming unresponsive during deployments because building Docker images on the server consumed all available resources (CPU and memory), causing SSH connection timeouts and failed deployments.

## Solution
Implemented a two-stage deployment pipeline:
1. **Build in CI**: Docker images are built on GitHub Actions runners (which have more resources)
2. **Pull on Deploy**: The droplet only pulls pre-built images from the registry

## Changes Made

### 1. CI Workflow (`ci.yml`)
Added a new `docker-push` job that:
- Runs after successful lint, test, and build jobs
- Only runs on `main` branch pushes
- Builds all Docker images using GitHub's infrastructure
- Pushes images to GitHub Container Registry (ghcr.io)
- Uses build caching for faster subsequent builds
- Tags images with both `latest` and commit SHA

Images built and pushed:
- `ghcr.io/ipcasj/ethhook-admin-api`
- `ghcr.io/ipcasj/ethhook-event-ingestor`
- `ghcr.io/ipcasj/ethhook-message-processor`
- `ghcr.io/ipcasj/ethhook-webhook-delivery`
- `ghcr.io/ipcasj/ethhook-ui`
- `ghcr.io/ipcasj/ethhook-demo-receiver`

### 2. Deployment Workflow (`digitalocean-deploy.yml`)
Updated deployment process:
- Logs into GitHub Container Registry
- Pulls pre-built images instead of building them
- Much faster and less resource-intensive
- Added `packages: read` permission to access registry

### 3. Docker Compose (`docker-compose.prod.yml`)
Changed all services from `build:` configuration to `image:` references:
```yaml
# Before:
build:
  context: .
  dockerfile: crates/admin-api/Dockerfile

# After:
image: ghcr.io/ipcasj/ethhook-admin-api:latest
```

## Benefits

1. **No More Resource Exhaustion**: Droplet no longer needs to compile Rust code
2. **Faster Deployments**: Pulling images takes seconds vs. minutes of building
3. **More Reliable**: No SSH timeouts from resource contention
4. **Better Caching**: GitHub Actions caches build layers across runs
5. **Versioning**: Images are tagged with commit SHAs for rollback capability
6. **Separation of Concerns**: Build environment separate from deployment environment

## Image Privacy
All images are stored in GitHub Container Registry and are private by default (same visibility as the repository). Only authenticated users with repository access can pull them.

## Next Steps
After the droplet is manually rebooted via the DigitalOcean control panel:
1. The next push to `main` will trigger the docker-push job
2. Images will be built and pushed to ghcr.io
3. Deployment workflow will pull and run those images
4. Monitor first deployment to ensure everything works correctly

## Rollback
If issues occur, you can:
1. Use a specific image tag: `ghcr.io/ipcasj/ethhook-admin-api:347a640`
2. Revert to the old build-on-deploy method by restoring the `build:` sections in docker-compose.prod.yml

## Monitoring
The docker-push job will appear in the CI workflow after the build job completes successfully on `main` branch pushes.
