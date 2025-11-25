# GitHub Personal Access Token Setup for Docker Registry

## Problem
GitHub Actions cannot push Docker images to `ghcr.io` for organization repositories using `GITHUB_TOKEN`. You need a Personal Access Token (PAT) with package write permissions.

## Solution: Create and Configure PAT

### Step 1: Create Personal Access Token

1. Go to GitHub: https://github.com/settings/tokens
2. Click **"Generate new token"** → **"Generate new token (classic)"**
3. Configure the token:
   - **Note**: `EthHook Docker Registry Access`
   - **Expiration**: Choose based on your needs (90 days recommended)
   - **Scopes**: Check these boxes:
     - ✅ `write:packages` - Upload packages to GitHub Package Registry
     - ✅ `read:packages` - Download packages from GitHub Package Registry
     - ✅ `delete:packages` - Delete packages from GitHub Package Registry (optional)
4. Click **"Generate token"**
5. **⚠️ IMPORTANT**: Copy the token immediately (you won't see it again!)

### Step 2: Add Token to Repository Secrets

1. Go to your repository: https://github.com/ipcasj/ethhook
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **"New repository secret"**
4. Configure the secret:
   - **Name**: `GH_PAT`
   - **Value**: Paste the token you copied in Step 1
5. Click **"Add secret"**

### Step 3: Verify Setup

After adding the secret, the next push will trigger CI and should successfully:
1. Build all Docker images ✅
2. Push images to `ghcr.io/ipcasj/ethhook-*` ✅
3. Make images available for deployment ✅

## Alternative: Make Repository Public

If you don't want to manage PAT tokens, you can make the repository public:

1. Go to **Settings** → **General**
2. Scroll to **Danger Zone** → **Change repository visibility**
3. Select **"Make public"**
4. Revert the workflow to use `GITHUB_TOKEN` (it works for public repos)

## Current Configuration

The CI workflow now uses `${{ secrets.GH_PAT }}` for authentication:

```yaml
- name: Log in to GitHub Container Registry
  uses: docker/login-action@v3
  with:
    registry: ghcr.io
    username: ${{ github.repository_owner }}
    password: ${{ secrets.GH_PAT }}  # ← Requires setup
```

## Testing

Once configured, test by pushing a commit:

```bash
git commit --allow-empty -m "Test Docker registry authentication"
git push origin main
```

Watch the CI run at: https://github.com/ipcasj/ethhook/actions

## Security Notes

- ✅ PAT tokens are more secure than password authentication
- ✅ Tokens can be revoked at any time
- ✅ Tokens have specific scopes (least privilege)
- ⚠️ Never commit tokens to version control
- ⚠️ Store tokens securely (use GitHub Secrets)
- ⚠️ Set expiration dates on tokens

## Troubleshooting

### Error: "Context access might be invalid: GH_PAT"
This is just a lint warning - it means the secret hasn't been added yet. Add the secret and it will work.

### Error: "denied: installation not allowed to Write organization package"
The current error - means you need to add the GH_PAT secret as described above.

### Error: "Bad credentials"
The PAT token is invalid or expired. Create a new one.

### Error: "Resource not accessible by integration"
The PAT token doesn't have the required scopes. Recreate with `write:packages` scope.

## Package Visibility

After first successful push, you may need to make packages public:

1. Go to https://github.com/orgs/ipcasj/packages
2. Find each package (ethhook-admin-api, ethhook-pipeline, ethhook-ui, ethhook-demo-receiver)
3. Click **Package settings** → **Change visibility** → **Public**

This allows the DigitalOcean server to pull images without authentication.

---

**Next Steps After Setup:**
1. Add GH_PAT secret ✅
2. Push this commit ✅
3. Watch CI complete successfully ✅
4. Verify images at https://github.com/orgs/ipcasj/packages ✅
5. Deploy to production ✅
