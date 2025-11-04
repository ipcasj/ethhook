# UI Testing Guide

## Problem
Working in "try-and-fix" mode is inefficient. We need quality proof before deployment.

## Testing Strategy

### 1. Build-Time Checks (Before Running)

#### TypeScript Type Checking
```bash
# Run TypeScript compiler in check mode
npm run build

# Or just type check without building
npx tsc --noEmit
```

**What it catches:**
- Undefined variables
- Missing properties
- Type mismatches
- Incorrect function signatures

**Example caught:**
```typescript
// ❌ Will fail type check
const maskSecret = (secret: string) => {
  return secret.substring(0, 8); // secret could be undefined!
};

// ✅ Correct
const maskSecret = (secret: string | undefined) => {
  if (!secret) return '••••••••••••••••••••••••••••••••';
  return secret.substring(0, 8);
};
```

#### ESLint
```bash
# Check for code quality issues
npm run lint

# Auto-fix issues
npm run lint -- --fix
```

**What it catches:**
- Unused variables
- Missing dependencies in useEffect
- Accessibility issues
- React best practices violations

### 2. Pre-Commit Checks (Automated)

Create `.husky/pre-commit`:
```bash
#!/usr/bin/env sh
. "$(dirname -- "$0")/_/husky.sh"

# Type check
echo "🔍 Running TypeScript check..."
npm run type-check || exit 1

# Lint
echo "🧹 Running ESLint..."
npm run lint || exit 1

# Build check
echo "🏗️  Testing build..."
npm run build || exit 1

echo "✅ All checks passed!"
```

Install:
```bash
npm install --save-dev husky
npx husky init
chmod +x .husky/pre-commit
```

### 3. Testing Checklist (Manual)

#### Before Starting Development
```bash
# 1. Clean install
cd ui
rm -rf node_modules .next
npm install

# 2. Type check
npx tsc --noEmit

# 3. Lint
npm run lint

# 4. Start dev server
npm run dev
```

#### After Making Changes
```bash
# 1. Type check
npx tsc --noEmit

# 2. Build check
npm run build

# 3. Test production build locally
npm start
```

#### Before Deployment
```bash
# 1. Full type check
npx tsc --noEmit

# 2. Production build
npm run build

# 3. Test Docker build
docker build -t ethhook-ui .
docker run -p 3002:3002 -e NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1 ethhook-ui

# 4. Test all pages manually
- http://localhost:3002/login
- http://localhost:3002/register
- http://localhost:3002/dashboard
- http://localhost:3002/dashboard/applications
- http://localhost:3002/dashboard/endpoints
- http://localhost:3002/dashboard/events
```

### 4. Common Issues & Prevention

#### Issue 1: Undefined Properties
**Error:** `Cannot read properties of undefined`

**Prevention:**
```typescript
// ❌ Bad - assumes data exists
const value = data.property.nested;

// ✅ Good - optional chaining
const value = data?.property?.nested;

// ✅ Good - with default
const value = data?.property?.nested ?? 'default';
```

#### Issue 2: Invalid Dates
**Error:** `Invalid time value`

**Prevention:**
```typescript
// ❌ Bad - no validation
return format(new Date(date), 'MMM d, yyyy');

// ✅ Good - validate first
export function formatDateTime(date: string | Date): string {
  if (!date) return 'N/A';
  const d = typeof date === 'string' ? new Date(date) : date;
  if (isNaN(d.getTime())) return 'Invalid date';
  return format(d, 'MMM d, yyyy HH:mm');
}
```

#### Issue 3: Missing Exports
**Error:** `Export X doesn't exist in target module`

**Prevention:**
```typescript
// Always export functions that are imported elsewhere
export function myFunction() { }  // ✅
function myFunction() { }         // ❌ Will fail if imported
```

#### Issue 4: API Type Mismatches
**Error:** Runtime errors when API returns unexpected data

**Prevention:**
```typescript
// Define strict types
interface Application {
  id: string;
  name: string;
  api_key?: string;  // Optional if might not exist
  created_at: string;
}

// Validate API responses
const { data } = useQuery<Application[]>({
  queryKey: ['applications'],
  queryFn: async () => {
    const response = await api.get<{ applications: Application[] }>('/applications');
    return response.applications || [];  // Provide fallback
  },
});
```

### 5. Quick Pre-Deploy Script

Create `scripts/pre-deploy.sh`:
```bash
#!/bin/bash

set -e  # Exit on error

echo "🔍 Step 1/5: Type checking..."
npx tsc --noEmit

echo "🧹 Step 2/5: Linting..."
npm run lint

echo "🏗️  Step 3/5: Building..."
npm run build

echo "🐳 Step 4/5: Testing Docker build..."
docker build -t ethhook-ui-test .

echo "✅ Step 5/5: All checks passed!"
echo "🚀 Ready to deploy!"
```

Usage:
```bash
chmod +x scripts/pre-deploy.sh
./scripts/pre-deploy.sh
```

### 6. CI/CD Integration (Recommended)

Add to `.github/workflows/ui-check.yml`:
```yaml
name: UI Quality Check

on:
  push:
    branches: [ main ]
    paths:
      - 'ui/**'
  pull_request:
    branches: [ main ]
    paths:
      - 'ui/**'

jobs:
  check:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '20'
        cache: 'npm'
        cache-dependency-path: ui/package-lock.json
    
    - name: Install dependencies
      working-directory: ui
      run: npm ci
    
    - name: Type check
      working-directory: ui
      run: npx tsc --noEmit
    
    - name: Lint
      working-directory: ui
      run: npm run lint
    
    - name: Build
      working-directory: ui
      run: npm run build
    
    - name: Docker build test
      working-directory: ui
      run: docker build -t ethhook-ui .
```

### 7. Testing Workflow (Step-by-Step)

#### Daily Development
1. **Start:** `npm run dev`
2. **Make changes**
3. **Check types:** `npx tsc --noEmit` (runs in ~2 seconds)
4. **See errors before runtime!**

#### Before Committing
1. `npx tsc --noEmit` - Type check
2. `npm run lint` - Code quality
3. `npm run build` - Ensure it builds
4. Commit

#### Before Deploying
1. Run `scripts/pre-deploy.sh`
2. Test Docker image locally
3. Manual smoke test (login, create, edit, delete)
4. Deploy

### 8. VS Code Integration

Add to `.vscode/settings.json`:
```json
{
  "typescript.tsdk": "node_modules/typescript/lib",
  "typescript.enablePromptUseWorkspaceTsdk": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "editor.formatOnSave": true,
  "eslint.validate": [
    "javascript",
    "javascriptreact",
    "typescript",
    "typescriptreact"
  ]
}
```

This gives you:
- ✅ Real-time type checking
- ✅ Inline error highlighting
- ✅ Auto-fix on save
- ✅ Import auto-complete

### 9. Common Commands Reference

```bash
# Quick checks (2-3 seconds each)
npx tsc --noEmit           # Type check
npm run lint               # Lint check

# Slower checks (20-30 seconds)
npm run build              # Full build

# Very slow (2-3 minutes)
docker build -t ethhook-ui . # Docker build
```

### 10. Quality Gates

#### ✅ Minimum (Required)
- TypeScript compiles with no errors
- ESLint passes
- Production build succeeds

#### 🎯 Recommended
- Pre-commit hooks enabled
- Docker build tested
- Manual smoke test before deploy

#### 🚀 Ideal
- CI/CD pipeline
- Automated E2E tests
- Visual regression testing

## Summary

**Fast feedback loop:**
1. `npx tsc --noEmit` after every change (2 seconds)
2. `npm run build` before committing (30 seconds)
3. Docker test before deploying (3 minutes)

**This prevents:**
- ❌ Runtime errors from undefined values
- ❌ Type mismatches
- ❌ Missing exports
- ❌ Invalid prop types
- ❌ Build failures in production

**Result:**
- ✅ Catch 90% of errors before running
- ✅ No more "try-and-fix" mode
- ✅ Confident deployments
- ✅ Professional development workflow
