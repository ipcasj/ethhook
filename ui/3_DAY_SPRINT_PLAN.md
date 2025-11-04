# EthHook UI - 3-Day Sprint Plan

## 🎯 Goal
Build production-ready Next.js UI with all core functionality in 72 hours.

---

## Day 1: Foundation (8 hours)

### Morning (4 hours)
- [x] **Setup** (30 min)
  - [x] Create Next.js project
  - [x] Install dependencies
  - [ ] Configure Tailwind
  - [ ] Setup shadcn/ui components

- [ ] **API Client** (1 hour)
  - [ ] Create API client wrapper
  - [ ] Setup TanStack Query provider
  - [ ] Configure environment variables
  - [ ] Test connection to Rust API

- [ ] **Authentication** (2.5 hours)
  - [ ] Login page UI
  - [ ] Register page UI
  - [ ] Auth context/hooks
  - [ ] Protected route middleware
  - [ ] JWT token handling

### Afternoon (4 hours)
- [ ] **Dashboard Layout** (2 hours)
  - [ ] Sidebar navigation
  - [ ] Top header with user menu
  - [ ] Mobile responsive layout
  - [ ] Dark mode toggle

- [ ] **Dashboard Page** (2 hours)
  - [ ] Metrics cards (total apps, endpoints, events)
  - [ ] Recent events table
  - [ ] Quick action buttons
  - [ ] Loading states

**Day 1 Deliverable:** Working auth + dashboard layout

---

## Day 2: CRUD Operations (8 hours)

### Morning (4 hours)
- [ ] **Applications Page** (2 hours)
  - [ ] List applications (table view)
  - [ ] Create application modal
  - [ ] Edit application inline
  - [ ] Delete confirmation dialog
  - [ ] API key display/copy

- [ ] **Endpoints Page - Part 1** (2 hours)
  - [ ] List endpoints (grouped by app)
  - [ ] Filter/search functionality
  - [ ] Create endpoint form (basic fields)

### Afternoon (4 hours)
- [ ] **Endpoints Page - Part 2** (3 hours)
  - [ ] Chain ID multi-select
  - [ ] Contract addresses input
  - [ ] Event signatures input
  - [ ] HMAC secret display/regenerate
  - [ ] Active/inactive toggle
  - [ ] Edit endpoint modal
  - [ ] Delete endpoint

- [ ] **Testing** (1 hour)
  - [ ] Test all CRUD operations
  - [ ] Fix bugs
  - [ ] Add loading/error states

**Day 2 Deliverable:** Full CRUD for applications + endpoints

---

## Day 3: Events + Polish (8 hours)

### Morning (4 hours)
- [ ] **Events Page** (3 hours)
  - [ ] Events list with filters
  - [ ] Real-time polling (every 3s)
  - [ ] Event detail drawer
  - [ ] JSON payload viewer
  - [ ] Delivery attempts timeline
  - [ ] Retry button for failed events
  - [ ] Status badges

- [ ] **Polish UI** (1 hour)
  - [ ] Add toast notifications
  - [ ] Improve loading states
  - [ ] Add empty states
  - [ ] Fix responsive issues

### Afternoon (4 hours)
- [ ] **Production Ready** (2 hours)
  - [ ] Environment configuration
  - [ ] Docker build
  - [ ] Update docker-compose
  - [ ] Test deployment flow

- [ ] **Documentation** (1 hour)
  - [ ] README for UI
  - [ ] Environment variables guide
  - [ ] Deployment instructions

- [ ] **Final Testing** (1 hour)
  - [ ] End-to-end testing
  - [ ] Cross-browser testing
  - [ ] Mobile testing
  - [ ] Fix critical bugs

**Day 3 Deliverable:** Production-ready MVP

---

## Tech Stack

### Core
- **Next.js 15**: React framework with App Router
- **TypeScript**: Type safety
- **Tailwind CSS**: Styling

### Libraries
- **@tanstack/react-query**: Data fetching/caching
- **recharts**: Charts for dashboard
- **date-fns**: Date formatting
- **lucide-react**: Icons

### UI Components (shadcn/ui)
- **button, card, input, label**: Form elements
- **table, badge, dialog**: Data display
- **dropdown-menu, tabs**: Navigation
- **toast, alert**: Notifications

---

## File Structure

```
ui/
├── app/
│   ├── (auth)/
│   │   ├── login/
│   │   │   └── page.tsx
│   │   └── register/
│   │       └── page.tsx
│   ├── (dashboard)/
│   │   ├── layout.tsx           # Dashboard layout with sidebar
│   │   ├── page.tsx              # Dashboard home
│   │   ├── applications/
│   │   │   └── page.tsx
│   │   ├── endpoints/
│   │   │   └── page.tsx
│   │   ├── events/
│   │   │   └── page.tsx
│   │   └── settings/
│   │       └── page.tsx
│   ├── api/
│   │   └── auth/
│   │       └── [...nextauth]/
│   │           └── route.ts
│   ├── layout.tsx                # Root layout
│   └── page.tsx                  # Landing page (redirect to dashboard)
├── components/
│   ├── ui/                       # shadcn/ui components
│   ├── dashboard/
│   │   ├── sidebar.tsx
│   │   ├── header.tsx
│   │   └── metrics-card.tsx
│   ├── applications/
│   │   ├── application-table.tsx
│   │   └── create-application-modal.tsx
│   ├── endpoints/
│   │   ├── endpoint-table.tsx
│   │   ├── endpoint-form.tsx
│   │   └── chain-selector.tsx
│   └── events/
│       ├── event-table.tsx
│       ├── event-detail.tsx
│       └── json-viewer.tsx
├── lib/
│   ├── api-client.ts             # Fetch wrapper for Rust API
│   ├── utils.ts                  # Utility functions
│   └── types.ts                  # TypeScript types
├── hooks/
│   ├── use-auth.ts               # Auth hook
│   ├── use-applications.ts       # Applications queries
│   ├── use-endpoints.ts          # Endpoints queries
│   └── use-events.ts             # Events queries
├── .env.local                    # Environment variables
├── .env.production              # Production env vars
└── Dockerfile                   # Production build
```

---

## API Integration

### Environment Variables
```bash
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:3000/api/v1
NEXT_PUBLIC_WS_URL=ws://localhost:3000/ws

# .env.production
NEXT_PUBLIC_API_URL=http://104.248.15.178:3000/api/v1
NEXT_PUBLIC_WS_URL=ws://104.248.15.178:3000/ws
```

### API Client Pattern
```typescript
// lib/api-client.ts
export async function apiClient<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const token = localStorage.getItem('token');
  
  const res = await fetch(`${process.env.NEXT_PUBLIC_API_URL}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
      ...options?.headers,
    },
  });

  if (!res.ok) {
    const error = await res.json();
    throw new Error(error.error || 'API error');
  }

  return res.json();
}
```

### React Query Hooks
```typescript
// hooks/use-applications.ts
export function useApplications() {
  return useQuery({
    queryKey: ['applications'],
    queryFn: () => apiClient<ApplicationListResponse>('/applications'),
    staleTime: 60 * 1000, // 1 minute
  });
}

export function useCreateApplication() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateApplicationRequest) =>
      apiClient('/applications', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] });
      toast.success('Application created');
    },
  });
}
```

---

## Key Features to Implement

### 1. Dashboard
- [x] Metrics: Total apps, endpoints, events (24h)
- [ ] Chart: Events over time (last 7 days)
- [ ] Chart: Delivery success rate
- [ ] Recent events list (last 10)
- [ ] Quick actions: Create app, Create endpoint

### 2. Applications
- [ ] CRUD operations (Create, Read, Update, Delete)
- [ ] API key display with copy button
- [ ] Regenerate API key
- [ ] Search/filter by name
- [ ] Sort by name, created date

### 3. Endpoints
- [ ] CRUD operations
- [ ] Multi-chain support (Ethereum, Polygon, Arbitrum, etc.)
- [ ] Contract address list (comma-separated)
- [ ] Event signature list (with validation)
- [ ] HMAC secret display/regenerate
- [ ] Active/inactive toggle
- [ ] Test endpoint button (send test event)

### 4. Events
- [ ] Real-time event list (polling every 3s)
- [ ] Filter by endpoint, status, date range
- [ ] Event detail modal:
  - Full JSON payload
  - Delivery attempts
  - Response headers/body
  - Retry button
- [ ] Status badges (delivered, failed, pending)
- [ ] Export to JSON/CSV

### 5. Settings
- [ ] User profile edit
- [ ] Password change
- [ ] Logout

---

## Performance Targets

- **Build Time**: < 30 seconds
- **Bundle Size**: < 300KB (gzipped)
- **First Contentful Paint**: < 1 second
- **Time to Interactive**: < 2 seconds
- **Lighthouse Score**: > 90

---

## Deployment

### Docker Build
```dockerfile
# ui/Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
ENV NEXT_PUBLIC_API_URL=http://104.248.15.178:3000/api/v1
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
EXPOSE 3002
CMD ["node", "server.js"]
```

### docker-compose.yml
```yaml
services:
  ui:
    build: ./ui
    ports:
      - "3002:3002"
    environment:
      - NEXT_PUBLIC_API_URL=http://admin-api:3000/api/v1
    networks:
      - ethhook-network
```

---

## Success Criteria

### Day 1 ✅
- [ ] Can login/register
- [ ] Can see dashboard with metrics
- [ ] Layout is responsive

### Day 2 ✅
- [ ] Can create/edit/delete applications
- [ ] Can create/edit/delete endpoints
- [ ] All forms validate properly

### Day 3 ✅
- [ ] Can view events in real-time
- [ ] Can retry failed deliveries
- [ ] Docker build works
- [ ] Deployed to Droplet

---

## Timeline

| Time | Task | Status |
|------|------|--------|
| **Day 1** | | |
| 08:00-08:30 | Setup project | ✅ Done |
| 08:30-09:30 | API client + TanStack Query | 🔄 Next |
| 09:30-12:00 | Authentication pages | ⏳ |
| 12:00-13:00 | Lunch break | |
| 13:00-15:00 | Dashboard layout | ⏳ |
| 15:00-17:00 | Dashboard page | ⏳ |
| 17:00-18:00 | Testing + fixes | ⏳ |
| **Day 2** | | |
| 08:00-10:00 | Applications CRUD | ⏳ |
| 10:00-12:00 | Endpoints create/list | ⏳ |
| 12:00-13:00 | Lunch break | |
| 13:00-16:00 | Endpoints edit/delete | ⏳ |
| 16:00-17:00 | Testing | ⏳ |
| 17:00-18:00 | Bug fixes | ⏳ |
| **Day 3** | | |
| 08:00-11:00 | Events page | ⏳ |
| 11:00-12:00 | UI polish | ⏳ |
| 12:00-13:00 | Lunch break | |
| 13:00-15:00 | Docker + deployment | ⏳ |
| 15:00-16:00 | Documentation | ⏳ |
| 16:00-18:00 | Testing + fixes | ⏳ |

---

## Next Steps

1. ✅ Project setup complete
2. 🔄 **NOW**: Create API client and configure environment
3. ⏳ Build login/register pages
4. ⏳ Build dashboard layout
5. ⏳ Continue per schedule...

**Estimated completion: 72 hours from now**
