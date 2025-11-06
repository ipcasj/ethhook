// TypeScript types matching your Rust API

export interface User {
  id: string;
  email: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  name: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Application {
  id: string;
  user_id: string;
  name: string;
  description: string | null;
  api_key: string;
  webhook_secret: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface ApplicationListResponse {
  applications: Application[];
  total: number;
}

export interface CreateApplicationRequest {
  name: string;
  description?: string;
}

export interface UpdateApplicationRequest {
  name?: string;
  description?: string;
  is_active?: boolean;
}

export interface Endpoint {
  id: string;
  application_id: string;
  name: string;
  webhook_url: string;
  description: string | null;
  hmac_secret: string;
  chain_ids: number[];
  contract_addresses: string[];
  event_signatures: string[];
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface EndpointListResponse {
  endpoints: Endpoint[];
  total: number;
}

export interface CreateEndpointRequest {
  application_id: string;
  name: string;
  webhook_url: string;
  description?: string;
  chain_ids: number[];
  contract_addresses: string[];
  event_signatures: string[];
}

export interface UpdateEndpointRequest {
  name?: string;
  webhook_url?: string;
  description?: string;
  chain_ids?: number[];
  contract_addresses?: string[];
  event_signatures?: string[];
  is_active?: boolean;
}

export interface Event {
  id: string;
  block_number: number;
  block_hash: string;
  transaction_hash: string;
  log_index: number;
  contract_address: string;
  topics: string[];
  data: string;
  ingested_at: string;
  processed_at: string | null;
  delivery_count: number | null;
  successful_deliveries: number | null;
  event_type: string;
  chain_id: number | null;
  endpoint_name: string | null;
  status: 'pending' | 'delivered' | 'failed';
  attempts: number;
  created_at: string;
}

export interface EventListResponse {
  events: Event[];
  total: number;
}

export interface DeliveryAttempt {
  id: string;
  event_id: string;
  attempt_number: number;
  status_code: number | null;
  response_body: string | null;
  response_headers: Record<string, string> | null;
  error_message: string | null;
  duration_ms: number | null;
  attempted_at: string;
}

export interface DashboardStats {
  events_today: number;
  events_total: number;
  success_rate: number;
  avg_delivery_time_ms: number | null;
  active_endpoints: number;
  total_deliveries: number;
  successful_deliveries: number;
  failed_deliveries: number;
}

export interface ErrorResponse {
  error: string;
}
