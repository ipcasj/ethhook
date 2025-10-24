# Start Frontend - Step by Step

The frontend isn't starting automatically. Here's how to start it manually:

## Option 1: Start from Terminal (Recommended)

Open a **new terminal window** and run:

```bash
cd /Users/igor/rust_projects/capstone0/crates/leptos-portal
trunk serve --port 3002
```

**Keep this terminal open!** You'll see:

```
📦 Starting build...
✅ Build succeeded!
📡 Serving on http://127.0.0.1:3002
```

Then open browser: **http://127.0.0.1:3002**

---

## Option 2: Check if Trunk is Installed

If you get "command not found: trunk":

```bash
# Check if trunk is installed
cargo install --list | grep trunk

# If not installed, install it:
cargo install --locked trunk

# Also install wasm target:
rustup target add wasm32-unknown-unknown
```

Then try Option 1 again.

---

## Option 3: Use the Admin API Directly (Temporary)

While frontend isn't working, you can use the API directly:

### Check System Health
```bash
curl http://localhost:8080/api/v1/health
```

### View Statistics (requires login first)
```bash
# First, register/login to get JWT token
# Then use that token for API calls
```

---

## Quick Fix: Start Everything

Run this script which handles the paths correctly:

```bash
cd /Users/igor/rust_projects/capstone0
./start_ui_demo.sh
```

This should:
1. Check services are running
2. Clean cache
3. Start trunk with correct path
4. Open browser automatically

---

## What URLs Should Work

Once frontend starts, these URLs should work:

```
http://127.0.0.1:3002          ← Frontend
http://localhost:3002           ← Same, different name
http://localhost:8080          ← Admin API (backend)
http://localhost:3001          ← Grafana
http://localhost:9090          ← Prometheus
```

---

## Current Status

Your backend is running perfectly:
- ✅ PostgreSQL running
- ✅ Redis running
- ✅ Event Ingestor running (capturing Sepolia events)
- ✅ Message Processor running
- ✅ Webhook Delivery running (sending to your receiver)
- ✅ Admin API running on port 8080
- ❌ Frontend needs manual start

**Just need to start the frontend manually!**

---

## Verify Backend is Working

Even without frontend, your system is working:

```bash
# Check backend health
curl http://localhost:8080/api/v1/health
# Should return: OK

# Check endpoints in database
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT name, is_active FROM endpoints WHERE is_active = true;
"

# Check if events are being captured
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT COUNT(*) as total_events FROM events;
"
```

Your webhook receiver terminal is showing webhooks = **system is working!**

Frontend is just the UI on top.

---

## Next Step

**Open a new terminal** and run:

```bash
cd /Users/igor/rust_projects/capstone0/crates/leptos-portal
~/.cargo/bin/trunk serve --port 3002
```

Or if trunk is in your PATH:

```bash
cd /Users/igor/rust_projects/capstone0/crates/leptos-portal
trunk serve --port 3002
```

**Then open**: http://127.0.0.1:3002

You should see the EthHook Portal! 🎉
