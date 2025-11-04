# EthHook UI

Modern Next.js frontend for the EthHook webhook platform.

## Features

- 🔐 **Authentication** - Login/Register with JWT
- 📱 **Applications** - Create and manage webhook applications
- 🔗 **Endpoints** - Configure multi-chain webhook endpoints  
- 📊 **Events** - Real-time event monitoring with filters
- 🎨 **Modern UI** - Built with shadcn/ui components
- ⚡ **Fast** - 20-30 second builds, ~200KB bundle size

## Tech Stack

- **Framework**: Next.js 16 (App Router)
- **Language**: TypeScript 5
- **Styling**: Tailwind CSS 4
- **UI Components**: shadcn/ui
- **Data Fetching**: TanStack Query (React Query)
- **Icons**: Lucide React
- **Toast Notifications**: Sonner

## Development

### Prerequisites

- Node.js 20+ 
- npm 10+

### Installation

```bash
npm install
```

### Environment Variables

Create a `.env.local` file:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

### Run Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

### Build for Production

```bash
npm run build
npm start
```

## Docker Deployment

### Build Docker Image

```bash
docker build -t ethhook-ui .
```

### Run Container

```bash
docker run -p 3000:3000 \
  -e NEXT_PUBLIC_API_URL=http://your-api-url/api/v1 \
  ethhook-ui
```

### Docker Compose

The UI is included in the main `docker-compose.prod.yml`:

```bash
# From project root
docker compose -f docker-compose.prod.yml up -d ui
```

The UI will be available on port 3002.

## Project Structure

```
ui/
├── app/
│   ├── (auth)/           # Authentication pages
│   │   ├── login/
│   │   └── register/
│   ├── (dashboard)/      # Dashboard pages
│   │   └── dashboard/
│   │       ├── applications/
│   │       ├── endpoints/
│   │       ├── events/
│   │       └── settings/
│   ├── layout.tsx        # Root layout
│   ├── page.tsx          # Home (redirects to login)
│   └── providers.tsx     # React Query provider
├── components/
│   └── ui/               # shadcn/ui components
├── lib/
│   ├── api-client.ts     # API client wrapper
│   ├── types.ts          # TypeScript types
│   └── utils.ts          # Utility functions
├── public/               # Static assets
└── Dockerfile           # Production Docker image
```

## API Integration

The UI connects to the EthHook Admin API:

### Authentication

- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user

### Applications

- `GET /api/v1/applications` - List applications
- `POST /api/v1/applications` - Create application
- `PUT /api/v1/applications/:id` - Update application
- `DELETE /api/v1/applications/:id` - Delete application

### Endpoints

- `GET /api/v1/endpoints` - List endpoints
- `POST /api/v1/endpoints` - Create endpoint
- `PUT /api/v1/endpoints/:id` - Update endpoint
- `DELETE /api/v1/endpoints/:id` - Delete endpoint

### Events

- `GET /api/v1/events` - List events (with filters)
- `GET /api/v1/stats` - Dashboard statistics

## Performance

- **Build Time**: 20-30 seconds
- **Bundle Size**: ~200KB (gzipped)
- **First Load**: ~2-3 seconds
- **Time to Interactive**: ~1 second

Compare to Leptos WASM:
- Build Time: 10-15 minutes
- Bundle Size: 1.5MB
- First Load: ~5-8 seconds

## License

MIT License - See LICENSE file in project root
