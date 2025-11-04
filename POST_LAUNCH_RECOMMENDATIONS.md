# Post-Launch Minor Recommendations

**Created:** November 2, 2025  
**Priority:** Backlog (Non-blocking)  
**Current Production Status:** ✅ Ready to Deploy

---

## Overview

These are **nice-to-have** enhancements that can be implemented after production launch. The UI is fully functional and production-ready without these items.

---

## Recommendations by Priority

### 🔴 High Priority (Week 1-2 Post-Launch)

#### 1. JWT Token Refresh & Session Management
**Effort:** 2-3 hours  
**Value:** Better user experience, reduced login friction

**Current Behavior:**
- ✅ Auto-detects expired tokens (401 errors)
- ✅ Auto-redirects to login page
- ✅ Clears invalid tokens

**Enhancements Needed:**

**A. Token Refresh Before Expiration**
```typescript
// lib/auth-refresh.ts
import { useEffect, useRef } from 'react';
import { api, setAuthToken, getAuthToken } from './api-client';
import { jwtDecode } from 'jwt-decode';

export function useTokenRefresh() {
  const refreshTimerRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    const token = getAuthToken();
    if (!token) return;

    try {
      const decoded: any = jwtDecode(token);
      const expiresAt = decoded.exp * 1000; // Convert to milliseconds
      const now = Date.now();
      const timeUntilExpiry = expiresAt - now;
      
      // Refresh 5 minutes before expiration
      const refreshTime = timeUntilExpiry - (5 * 60 * 1000);
      
      if (refreshTime > 0) {
        refreshTimerRef.current = setTimeout(async () => {
          try {
            const response = await api.post('/auth/refresh');
            setAuthToken(response.token);
          } catch (error) {
            console.error('Token refresh failed:', error);
          }
        }, refreshTime);
      }
    } catch (error) {
      console.error('Failed to decode token:', error);
    }

    return () => {
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current);
      }
    };
  }, []);
}
```

**B. Session Expired Toast Notification**
```typescript
// Update api-client.ts
if (response.status === 401) {
  clearAuthToken();
  
  // Show toast notification
  if (typeof window !== 'undefined') {
    const { toast } = await import('sonner');
    toast.error('Your session has expired. Please log in again.');
    
    setTimeout(() => {
      window.location.href = '/login';
    }, 2000); // 2 second delay for user to see message
  }
}
```

**C. Silent Background Renewal**
```typescript
// Add to root layout
'use client';

import { useTokenRefresh } from '@/lib/auth-refresh';

export function AuthProvider({ children }) {
  useTokenRefresh(); // Automatically refresh token in background
  
  return <>{children}</>;
}
```

**Implementation Steps:**
1. [ ] Install `jwt-decode`: `npm install jwt-decode`
2. [ ] Create `lib/auth-refresh.ts` with token refresh hook
3. [ ] Update `api-client.ts` to show toast before redirect
4. [ ] Add `AuthProvider` to root layout
5. [ ] Test token refresh flow
6. [ ] Verify no interruption to user workflow

**Backend Requirement:**
- Ensure `/auth/refresh` endpoint exists in admin-api
- Token refresh should extend expiration without re-login

---

#### 2. Complete E2E Test Migration
**Effort:** 1-2 hours  
**Value:** High confidence in future changes

**Tasks:**
- [ ] Update `ui/e2e/01-auth.spec.ts` to use `data-testid` selectors
  - Replace `input#email` with `[data-testid="email-input"]`
  - Replace `input#password` with `[data-testid="password-input"]`
  - Replace `input[name="email"]` with `[data-testid="email-input"]`
  - Verify all auth flows work

- [ ] Review `ui/e2e/02-applications.spec.ts` for consistency
  - Ensure all selectors use `data-testid` pattern
  - Test CRUD operations
  - Verify empty states

- [ ] Review `ui/e2e/03-endpoints.spec.ts` for consistency
  - Ensure all selectors use `data-testid` pattern
  - Test endpoint creation with all chain options
  - Verify contract address and event signature handling

**Current Status:**
- ✅ Smoke tests (00-smoke.spec.ts) - 3/3 passing
- ⚠️ Auth tests - need selector updates
- ⚠️ App tests - need review
- ⚠️ Endpoint tests - partially updated

**Success Criteria:**
```bash
npm run test:e2e  # All tests pass
```

---

#### 2. Error Tracking Setup
**Effort:** 1 hour  
**Value:** Critical for production monitoring

**Recommended Tools:**
- **Sentry** (Recommended) - Best for React/Next.js
- **LogRocket** - Session replay + error tracking
- **Rollbar** - Lightweight alternative

**Implementation (Sentry Example):**
```bash
cd ui
npm install @sentry/nextjs
npx @sentry/wizard@latest -i nextjs
```

**Configuration:**
```typescript
// sentry.client.config.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NODE_ENV,
  tracesSampleRate: 0.1,
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,
});
```

**Metrics to Track:**
- JavaScript errors and stack traces
- API request failures
- Failed authentication attempts
- Form validation errors
- Page load times

---

#### 3. Basic Analytics
**Effort:** 30 minutes  
**Value:** Understand user behavior

**Recommended Tools:**
- **Vercel Analytics** (if deploying to Vercel) - Zero config
- **Google Analytics 4** - Industry standard
- **Plausible** - Privacy-focused alternative

**Implementation (Vercel Analytics):**
```bash
npm install @vercel/analytics
```

```tsx
// app/layout.tsx
import { Analytics } from '@vercel/analytics/react';

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  );
}
```

**Key Metrics:**
- Page views by route
- User session duration
- Feature adoption (apps created, endpoints configured)
- Conversion funnel (registration → first app → first endpoint)

---

### 🟡 Medium Priority (Month 1 Post-Launch)

#### 4. React Error Boundaries
**Effort:** 30 minutes  
**Value:** Better error recovery UX

**Files to Create:**

**`ui/app/error.tsx`** (App-level error boundary)
```tsx
'use client';

import { useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { AlertCircle } from 'lucide-react';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // Log to error tracking service
    console.error('Application error:', error);
  }, [error]);

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <Card className="max-w-md w-full p-6 text-center">
        <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
        <h2 className="text-2xl font-bold mb-2">Something went wrong</h2>
        <p className="text-muted-foreground mb-4">
          We're sorry, but something unexpected happened.
        </p>
        <Button
          onClick={reset}
          className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700"
        >
          Try Again
        </Button>
      </Card>
    </div>
  );
}
```

**`ui/app/(dashboard)/dashboard/error.tsx`** (Dashboard-level)
```tsx
'use client';

import { useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { AlertCircle } from 'lucide-react';

export default function DashboardError({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  useEffect(() => {
    console.error('Dashboard error:', error);
  }, [error]);

  return (
    <div className="flex flex-col items-center justify-center py-12">
      <AlertCircle className="w-16 h-16 text-red-500 mb-4" />
      <h2 className="text-xl font-bold mb-2">Failed to load dashboard</h2>
      <p className="text-muted-foreground mb-4">{error.message}</p>
      <Button onClick={reset}>Reload</Button>
    </div>
  );
}
```

**Testing:**
- Simulate errors in development
- Verify error boundary catches and displays
- Test reset functionality

---

#### 5. Loading Skeletons
**Effort:** 1-2 hours  
**Value:** Better perceived performance

**Create Skeleton Components:**

**`ui/components/ui/skeleton.tsx`**
```tsx
import { cn } from '@/lib/utils';

export function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn('animate-pulse rounded-md bg-slate-200', className)}
      {...props}
    />
  );
}
```

**Apply to Pages:**

```tsx
// Example: Applications page loading state
{isLoading ? (
  <div className="space-y-4">
    {[1, 2, 3].map((i) => (
      <Card key={i} className="p-6">
        <Skeleton className="h-4 w-1/4 mb-2" />
        <Skeleton className="h-8 w-1/2" />
      </Card>
    ))}
  </div>
) : (
  // ... actual content
)}
```

**Pages to Update:**
- Applications list
- Endpoints list
- Events list
- Dashboard statistics
- Settings profile

---

#### 6. Settings Page E2E Tests
**Effort:** 30 minutes  
**Value:** Ensure profile editing works

**Create:** `ui/e2e/04-settings.spec.ts`
```typescript
import { test, expect } from '@playwright/test';
import { login } from './fixtures/test-helpers';

test.describe('Settings Page', () => {
  test('should display user profile', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await login(page, 'demo@ethhook.com', 'Demo1234!');
    
    await page.click('text=Settings');
    await expect(page.locator('h1:has-text("Settings")')).toBeVisible();
    
    // Verify email is read-only
    const emailInput = page.locator('input[type="email"]');
    await expect(emailInput).toBeDisabled();
    await expect(emailInput).toHaveValue('demo@ethhook.com');
  });

  test('should edit user name', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await login(page, 'demo@ethhook.com', 'Demo1234!');
    
    await page.click('text=Settings');
    await page.click('button:has-text("Edit Profile")');
    
    const nameInput = page.locator('[data-testid="name-input"]');
    await nameInput.clear();
    await nameInput.fill('Updated Demo User');
    
    await page.click('button:has-text("Save Changes")');
    await expect(page.locator('text=Profile updated successfully')).toBeVisible();
  });
});
```

---

### 🟢 Low Priority (Quarter 1)

#### 7. Mobile Navigation Menu
**Effort:** 1-2 hours  
**Value:** Better mobile UX

**Install Sheet Component:**
```bash
npx shadcn-ui@latest add sheet
```

**Implementation:**
```tsx
// components/mobile-nav.tsx
'use client';

import { useState } from 'react';
import { Menu } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet';
import { Sidebar } from './sidebar';

export function MobileNav() {
  const [open, setOpen] = useState(false);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild className="lg:hidden">
        <Button variant="ghost" size="icon">
          <Menu className="h-6 w-6" />
        </Button>
      </SheetTrigger>
      <SheetContent side="left" className="p-0 w-64">
        <Sidebar onNavigate={() => setOpen(false)} />
      </SheetContent>
    </Sheet>
  );
}
```

**Add to Layout:**
```tsx
// app/(dashboard)/layout.tsx
<div className="lg:hidden">
  <MobileNav />
</div>
```

---

#### 8. Keyboard Shortcuts
**Effort:** 1 hour  
**Value:** Power user productivity

**Implementation:**
```tsx
// hooks/use-keyboard-shortcuts.ts
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';

export function useKeyboardShortcuts() {
  const router = useRouter();

  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Command/Ctrl + K for quick search
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        // Open command palette
      }

      // G shortcuts for navigation (like GitHub)
      if (e.key === 'g') {
        const nextKey = await waitForNextKey();
        if (nextKey === 'd') router.push('/dashboard');
        if (nextKey === 'a') router.push('/dashboard/applications');
        if (nextKey === 'e') router.push('/dashboard/endpoints');
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [router]);
}
```

**Shortcuts to Implement:**
- `G` + `D` → Dashboard
- `G` + `A` → Applications
- `G` + `E` → Endpoints
- `G` + `V` → Events (view)
- `G` + `S` → Settings
- `Cmd/Ctrl` + `K` → Command palette
- `C` → Create (context-aware)
- `?` → Show keyboard shortcuts help

---

#### 9. Dark Mode (Infrastructure Exists)
**Effort:** 2 hours  
**Value:** User preference

**Current Status:**
- ✅ Tailwind configured with dark mode
- ✅ `dark:` classes present in some components
- ⚠️ Not fully implemented across all pages

**Tasks:**
- [ ] Add theme toggle component
- [ ] Implement theme provider
- [ ] Update all components with dark variants
- [ ] Store preference in localStorage
- [ ] Test all pages in dark mode

**Implementation:**
```bash
npx shadcn-ui@latest add dropdown-menu
```

```tsx
// components/theme-toggle.tsx
'use client';

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { Button } from '@/components/ui/button';

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
    >
      <Sun className="h-5 w-5 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
      <Moon className="absolute h-5 w-5 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
    </Button>
  );
}
```

---

#### 10. Feature Tour / Onboarding
**Effort:** 2-3 hours  
**Value:** Better first-time user experience

**Recommended Library:**
- **Shepherd.js** - Lightweight tour library
- **Intro.js** - Popular alternative
- **React Joyride** - React-specific

**Implementation:**
```bash
npm install react-joyride
```

```tsx
// components/onboarding-tour.tsx
'use client';

import { useState, useEffect } from 'react';
import Joyride from 'react-joyride';

export function OnboardingTour() {
  const [run, setRun] = useState(false);

  useEffect(() => {
    // Check if user has seen tour
    const hasSeenTour = localStorage.getItem('hasSeenTour');
    if (!hasSeenTour) {
      setRun(true);
    }
  }, []);

  const steps = [
    {
      target: '[data-tour="dashboard"]',
      content: 'Welcome! This is your dashboard overview.',
    },
    {
      target: '[data-tour="create-app"]',
      content: 'Start by creating your first application.',
    },
    {
      target: '[data-tour="create-endpoint"]',
      content: 'Then configure an endpoint to receive events.',
    },
  ];

  return (
    <Joyride
      steps={steps}
      run={run}
      continuous
      showSkipButton
      callback={(data) => {
        if (data.status === 'finished' || data.status === 'skipped') {
          localStorage.setItem('hasSeenTour', 'true');
          setRun(false);
        }
      }}
    />
  );
}
```

---

## Implementation Schedule

### Week 1-2 Post-Launch
- [ ] Complete E2E test migration (2 hours)
- [ ] Setup Sentry error tracking (1 hour)
- [ ] Add Vercel Analytics (30 min)
- [ ] Monitor production metrics daily

### Month 1
- [ ] Implement React Error Boundaries (30 min)
- [ ] Add loading skeletons (2 hours)
- [ ] Create Settings E2E tests (30 min)
- [ ] Review and prioritize based on user feedback

### Quarter 1
- [ ] Mobile navigation improvements (2 hours)
- [ ] Keyboard shortcuts (1 hour)
- [ ] Complete dark mode (2 hours)
- [ ] Feature tour for onboarding (3 hours)

---

## Success Metrics

Track these after implementing each recommendation:

### E2E Tests
- ✅ All test files passing
- ✅ Test coverage > 80%
- ✅ Tests run in CI/CD pipeline

### Error Tracking
- 📊 Error rate < 1%
- 📊 Mean time to resolution (MTTR) < 24 hours
- 📊 Zero critical errors

### Analytics
- 📊 User retention rate
- 📊 Feature adoption rates
- 📊 Time to first endpoint created
- 📊 Active user count

### UX Improvements
- 📊 Reduced bounce rate on mobile
- 📊 Increased session duration
- 📊 Positive user feedback scores

---

## Notes

- All recommendations are **non-blocking** for production launch
- Prioritize based on actual user feedback and usage patterns
- Some items may become unnecessary if not requested by users
- Budget ~2-3 hours per week for post-launch improvements
- Monitor production for 2 weeks before implementing changes

---

**Status:** 📋 Backlog  
**Owner:** Development Team  
**Review Date:** 2 weeks post-launch
