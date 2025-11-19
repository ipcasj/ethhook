# GitHub Secrets Configuration Checklist

## Required Secrets for Deployment

The deployment workflow requires the following GitHub secrets to be configured at:
`https://github.com/ipcasj/ethhook/settings/secrets/actions`

### 1. DROPLET_HOST
- **Description:** IP address of the DigitalOcean droplet
- **Example:** `104.248.15.178`
- **How to get:** From DigitalOcean dashboard → Droplets → Your droplet
- **Status:** ✅ Should be set to: `104.248.15.178` (from workflow comments)

### 2. DROPLET_USER
- **Description:** SSH username for the droplet
- **Example:** `root` or your username
- **How to get:** Usually `root` for DigitalOcean droplets, or the username you created
- **Status:** ⚠️ Verify this is set

### 3. DROPLET_SSH_KEY
- **Description:** Private SSH key for authentication (full key content)
- **Format:** Multi-line string starting with `-----BEGIN OPENSSH PRIVATE KEY-----`
- **How to get:** 
  - From your local machine: `cat ~/.ssh/id_rsa` or `cat ~/.ssh/id_ed25519`
  - Or from the key you created specifically for GitHub Actions
- **Important:** 
  - Must include the BEGIN and END markers
  - Must be the **private key**, not the public key
  - Should match the public key installed on the droplet
- **Status:** ⚠️ Verify this is set correctly

### 4. PRODUCTION_URL (Optional)
- **Description:** URL of the production environment (for deployment summary)
- **Example:** `http://104.248.15.178:3002`
- **Status:** Optional, used only for display

## How to Verify Secrets

1. Go to: https://github.com/ipcasj/ethhook/settings/secrets/actions
2. Check that all three required secrets appear in the list
3. You cannot view secret values, but you can:
   - Update/replace them if needed
   - Delete and recreate them if corrupted

## How to Set Up SSH Key

If you need to create a new SSH key pair for deployment:

```bash
# 1. Generate new key pair (on your local machine)
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_deploy_key

# 2. Copy the PUBLIC key to the droplet
ssh-copy-id -i ~/.ssh/github_deploy_key.pub root@104.248.15.178

# OR manually add it to the droplet:
cat ~/.ssh/github_deploy_key.pub
# Then SSH into droplet and add to ~/.ssh/authorized_keys

# 3. Copy the PRIVATE key content for GitHub secret
cat ~/.ssh/github_deploy_key
# Copy the ENTIRE output (including BEGIN/END markers)

# 4. Add to GitHub:
# - Go to: https://github.com/ipcasj/ethhook/settings/secrets/actions
# - Click "New repository secret"
# - Name: DROPLET_SSH_KEY
# - Value: Paste the private key
# - Click "Add secret"
```

## Testing the Setup

After configuring secrets, test with:
1. Go to: https://github.com/ipcasj/ethhook/actions/workflows/digitalocean-deploy.yml
2. Click "Run workflow" → "Run workflow" (manual trigger)
3. Watch the workflow run
4. The new "Validate Required Secrets" step will show:
   - ✅ Which secrets are configured
   - ❌ Which secrets are missing

## Common Issues

### Issue: "Process completed with exit code 1" in Setup SSH key step
**Cause:** One or more secrets are not set or contain invalid data

**Solution:**
1. Check the "Validate Required Secrets" step output (added in latest workflow)
2. Verify each secret is set in GitHub
3. For SSH key, ensure you copied the ENTIRE private key including headers

### Issue: SSH connection fails
**Cause:** Public key not installed on droplet, or wrong user

**Solution:**
1. SSH manually to test: `ssh -i ~/.ssh/your_key root@104.248.15.178`
2. Verify the public key is in `~/.ssh/authorized_keys` on the droplet
3. Check that DROPLET_USER matches the actual username

### Issue: "Host key verification failed"
**Cause:** Droplet fingerprint not in known_hosts (usually auto-fixed by workflow)

**Solution:**
1. The workflow runs `ssh-keyscan` to auto-add the host key
2. If still failing, manually run: `ssh-keyscan -H 104.248.15.178 >> ~/.ssh/known_hosts`

## Troubleshooting Checklist

- [ ] All secrets are set in GitHub Actions secrets page
- [ ] SSH key is the PRIVATE key (not public)
- [ ] SSH key includes BEGIN/END markers
- [ ] Public key is installed on the droplet in `~/.ssh/authorized_keys`
- [ ] DROPLET_USER matches actual username on droplet
- [ ] DROPLET_HOST is the correct IP address
- [ ] Can manually SSH to droplet: `ssh -i ~/.ssh/key user@host`
- [ ] Workflow has latest validation step that shows clear error messages

## What Changed?

The deployment was working yesterday. Recent commits show only formatting changes (whitespace) to workflow files, so the issue is likely:

1. **Secrets expired or were deleted** - Check if they still exist in GitHub
2. **Droplet was recreated** - New IP or SSH keys would break the connection
3. **SSH key permissions** - The key might have been regenerated

**Action:** Verify all three secrets still exist and contain valid values at:
https://github.com/ipcasj/ethhook/settings/secrets/actions
