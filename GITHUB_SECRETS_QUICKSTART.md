# Quick Setup: GitHub Secrets for Deployment

You already have SSH access to your DigitalOcean droplet! This guide shows you how to copy your existing SSH key to GitHub so automated deployments work.

## 🔑 Step 1: Copy Your SSH Private Key

Run this command to display your private key:

```bash
cat ~/.ssh/id_rsa
```

**Copy the entire output**, including these lines:
```
-----BEGIN RSA PRIVATE KEY-----
...all the lines in between...
-----END RSA PRIVATE KEY-----
```

## 🚀 Step 2: Add Secrets to GitHub

1. Go to: https://github.com/ipcasj/ethhook/settings/secrets/actions
2. Click **"New repository secret"** button
3. Add these 4 secrets:

### Secret 1: `DROPLET_HOST`
- **Name:** `DROPLET_HOST`
- **Value:** `104.248.15.178`

### Secret 2: `DROPLET_USER`
- **Name:** `DROPLET_USER`  
- **Value:** `root`

### Secret 3: `DROPLET_SSH_KEY`
- **Name:** `DROPLET_SSH_KEY`
- **Value:** Paste the entire private key from Step 1 (the output of `cat ~/.ssh/id_rsa`)

### Secret 4: `PRODUCTION_URL`
- **Name:** `PRODUCTION_URL`
- **Value:** `http://104.248.15.178:3000` (backend API URL for health checks)

## ✅ Step 3: Test It

After adding all secrets, test the deployment:

```bash
# Make a small change
echo "# Test deployment" >> README.md
git add README.md
git commit -m "test: Verify automatic deployment"
git push origin main
```

Then watch it deploy automatically at:
https://github.com/ipcasj/ethhook/actions

## 🔒 Security Notes

- Your private key stays encrypted in GitHub (only GitHub Actions can use it)
- The key is never exposed in logs
- You can rotate/change the key anytime by updating the secret
- The key is the same one you're already using - nothing new to create!

## 🆘 If You Get Locked Out

If you accidentally break SSH access:
1. Log into DigitalOcean dashboard
2. Use the web console to access your droplet
3. Fix the SSH keys in `~/.ssh/authorized_keys`

## 💡 What This Does

Once configured, **every push to `main` branch** will:
1. SSH into your droplet (using the key you just added)
2. Pull latest code
3. Rebuild Docker containers
4. Run database migrations
5. Restart services
6. Health check to verify everything works

No manual deployment needed! 🎉
