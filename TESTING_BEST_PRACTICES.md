# Testing Best Practices for EthHook

## The Problem We Had

Tests were breaking because they assumed form fields would have `name` attributes, but our shadcn/ui components use `id` attributes. This created hours of debugging.

## The Solution: Use `data-testid`

Add `data-testid` attributes to all interactive elements. This creates a **stable testing API** independent of implementation details.

### Example: Login Form

**Before (fragile):**
```tsx
<Input
  id="email"
  type="email"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
/>
```

**After (test-friendly):**
```tsx
<Input
  id="email"
  type="email"
  data-testid="email-input"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
/>
```

**Test becomes simple:**
```typescript
await page.fill('[data-testid="email-input"]', 'user@example.com');
```

## Quick Wins

### 1. Add to All Form Inputs

```tsx
// Login/Register
<Input data-testid="email-input" />
<Input data-testid="password-input" />
<Input data-testid="name-input" />

// Applications
<Input data-testid="app-name-input" />
<Input data-testid="app-description-input" />

// Endpoints
<Input data-testid="webhook-url-input" />
```

### 2. Add to Buttons

```tsx
<Button data-testid="submit-button">Submit</Button>
<Button data-testid="create-app-button">Create Application</Button>
<Button data-testid="add-endpoint-button">Add Endpoint</Button>
```

### 3. Add to Interactive Elements

```tsx
<Select data-testid="chain-select">
<Checkbox data-testid="chain-ethereum" />
<Dialog data-testid="create-dialog">
```

## Benefits

✅ **Tests never break from UI refactoring** - Change CSS, IDs, classes freely  
✅ **Clear testing intent** - `data-testid="submit-button"` is obvious  
✅ **Industry standard** - Works with Playwright, Cypress, Testing Library  
✅ **Fast debugging** - Know exactly which element the test is looking for  
✅ **Production-safe** - Can strip `data-testid` in prod builds if needed  

## Implementation Plan

### Phase 1: Critical Paths (30 min)
1. Auth forms: login, register (4 inputs)
2. Application CRUD (2 inputs + 1 button)
3. Endpoint CRUD (1 input + 1 button)

### Phase 2: Update Test Helpers (15 min)
Update `test-helpers.ts` to prefer `data-testid`:

```typescript
async fillField(testId: string, value: string) {
  await this.page.fill(`[data-testid="${testId}"]`, value);
}
```

### Phase 3: Run Tests (5 min)
All tests should pass without touching 32 test cases.

## Result

**Instead of fixing 50+ selectors across 4 test files, you fix ~15 UI components once.**

Tests become resilient and your UI can evolve freely.
