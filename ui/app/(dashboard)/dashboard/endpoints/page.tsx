'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { StatusBadge } from '@/components/ui/status-badge';
import { InfoBanner } from '@/components/ui/info-banner';
import { api } from '@/lib/api-client';
import { Endpoint, EndpointListResponse, ApplicationListResponse } from '@/lib/types';
import { Webhook, Plus, Edit, Trash2, Copy, BarChart3 } from 'lucide-react';
import { formatDateTime, copyToClipboard, truncate } from '@/lib/utils';
import { toast } from 'sonner';

const CHAIN_OPTIONS = [
  { value: 1, label: 'Ethereum Mainnet' },
  { value: 11155111, label: 'Sepolia Testnet' },
  { value: 137, label: 'Polygon' },
  { value: 42161, label: 'Arbitrum' },
  { value: 10, label: 'Optimism' },
  { value: 8453, label: 'Base' },
];

export default function EndpointsPage() {
  const router = useRouter();
  const queryClient = useQueryClient();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [selectedEndpoint, setSelectedEndpoint] = useState<Endpoint | null>(null);
  
  // Form state
  const [applicationId, setApplicationId] = useState('');
  const [name, setName] = useState('');
  const [webhookUrl, setWebhookUrl] = useState('');
  const [chainIds, setChainIds] = useState<number[]>([]);
  const [contractAddresses, setContractAddresses] = useState('');
  const [eventSignatures, setEventSignatures] = useState('');

  // Fetch endpoints
  const { data: endpointsData, isLoading: endpointsLoading } = useQuery<EndpointListResponse>({
    queryKey: ['endpoints'],
    queryFn: () => api.get<EndpointListResponse>('/endpoints'),
  });

  // Fetch applications for dropdown
  const { data: appsData } = useQuery<ApplicationListResponse>({
    queryKey: ['applications'],
    queryFn: () => api.get<ApplicationListResponse>('/applications'),
  });

  // Create mutation
  const createMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => api.post<Endpoint>('/endpoints', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['endpoints'] });
      resetForm();
      setIsCreateOpen(false);
      toast.success('Endpoint created successfully!');
    },
    onError: (error) => {
      const errorMessage = error instanceof Error ? error.message : 'Failed to create endpoint';
      toast.error(errorMessage);
    },
  });

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Record<string, unknown> }) =>
      api.put<Endpoint>(`/endpoints/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['endpoints'] });
      resetForm();
      setIsEditOpen(false);
      setSelectedEndpoint(null);
      toast.success('Endpoint updated successfully!');
    },
    onError: (error) => {
      const errorMessage = error instanceof Error ? error.message : 'Failed to update endpoint';
      toast.error(errorMessage);
    },
  });

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/endpoints/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['endpoints'] });
      toast.success('Endpoint deleted successfully!');
    },
    onError: (error) => {
      const errorMessage = error instanceof Error ? error.message : 'Failed to delete endpoint';
      toast.error(errorMessage);
    },
  });

  const resetForm = () => {
    setApplicationId('');
    setName('');
    setWebhookUrl('');
    setChainIds([]);
    setContractAddresses('');
    setEventSignatures('');
  };

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    const addresses = contractAddresses.split(',').map(a => a.trim()).filter(Boolean);
    const signatures = eventSignatures.split(',').map(s => s.trim()).filter(Boolean);
    
    createMutation.mutate({
      application_id: applicationId,
      name,
      webhook_url: webhookUrl,
      chain_ids: chainIds,
      contract_addresses: addresses,
      event_signatures: signatures,
    });
  };

  const handleEdit = (e: React.FormEvent) => {
    e.preventDefault();
    if (selectedEndpoint) {
      const addresses = contractAddresses.split(',').map(a => a.trim()).filter(Boolean);
      const signatures = eventSignatures.split(',').map(s => s.trim()).filter(Boolean);
      
      updateMutation.mutate({
        id: selectedEndpoint.id,
        data: {
          name,
          webhook_url: webhookUrl,
          chain_ids: chainIds,
          contract_addresses: addresses,
          event_signatures: signatures,
        },
      });
    }
  };

  const handleDelete = (endpoint: Endpoint) => {
    if (confirm(`Are you sure you want to delete "${endpoint.name}"? This action cannot be undone.`)) {
      deleteMutation.mutate(endpoint.id);
    }
  };

  const openEditDialog = (endpoint: Endpoint) => {
    setSelectedEndpoint(endpoint);
    setName(endpoint.name);
    setWebhookUrl(endpoint.webhook_url);
    setChainIds(endpoint.chain_ids);
    setContractAddresses(endpoint.contract_addresses.join(', '));
    setEventSignatures(endpoint.event_signatures.join(', '));
    setIsEditOpen(true);
  };

  const toggleChain = (chainId: number) => {
    setChainIds(prev =>
      prev.includes(chainId)
        ? prev.filter(id => id !== chainId)
        : [...prev, chainId]
    );
  };

  const getAppName = (appId: string) => {
    return appsData?.applications.find(app => app.id === appId)?.name || 'Unknown';
  };

  const getChainName = (chainId: number) => {
    return CHAIN_OPTIONS.find(c => c.value === chainId)?.label || String(chainId);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Endpoints</h1>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            Configure webhook endpoints for your applications
          </p>
        </div>
        <Button 
          onClick={() => setIsCreateOpen(true)} 
          data-testid="add-endpoint-button"
          className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30"
        >
          <Plus className="w-4 h-4 mr-2" />
          Add Endpoint
        </Button>
      </div>

      <InfoBanner
        title="Manage Your Webhook Endpoints"
        description="Endpoints receive notifications when specific blockchain events occur. Configure chains, contracts, and event signatures to filter the events you want to monitor."
        tips={[
          'Test endpoints with the "Test" button to verify connectivity and authentication',
          'Use multiple chain IDs to monitor the same events across different blockchain networks',
          'Specify contract addresses to filter events from specific smart contracts',
          'Event signatures (topics[0]) allow you to narrow down to specific event types like Transfer or Approval'
        ]}
        defaultCollapsed={true}
      />

      <Card>
        <CardHeader>
          <CardTitle>Your Endpoints</CardTitle>
          <CardDescription>Webhook endpoints that receive blockchain events</CardDescription>
        </CardHeader>
        <CardContent>
          {endpointsLoading ? (
            <div className="text-center py-12">
              <p className="text-muted-foreground">Loading endpoints...</p>
            </div>
          ) : !endpointsData?.endpoints || endpointsData.endpoints.length === 0 ? (
            <div className="text-center py-12">
              <Webhook className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
              <p className="text-muted-foreground">No endpoints configured</p>
              <p className="text-sm text-muted-foreground mt-1">
                Add an endpoint to start receiving events from the blockchain
              </p>
              <Button 
                className="mt-4 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30" 
                onClick={() => setIsCreateOpen(true)}
              >
                <Plus className="w-4 h-4 mr-2" />
                Add Endpoint
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Application</TableHead>
                  <TableHead>Webhook URL</TableHead>
                  <TableHead>Chains</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {endpointsData.endpoints.map((endpoint) => (
                  <TableRow key={endpoint.id}>
                    <TableCell className="font-medium">{endpoint.name}</TableCell>
                    <TableCell className="text-muted-foreground">
                      {getAppName(endpoint.application_id)}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <code className="text-xs bg-muted px-2 py-1 rounded max-w-xs truncate">
                          {truncate(endpoint.webhook_url, 40)}
                        </code>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={async (e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            console.log('Copying URL:', endpoint.webhook_url);
                            const success = await copyToClipboard(endpoint.webhook_url);
                            if (success) {
                              toast.success('URL copied to clipboard!');
                            } else {
                              toast.error('Failed to copy URL');
                            }
                          }}
                          title={endpoint.webhook_url}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {endpoint.chain_ids.slice(0, 2).map(chainId => (
                          <Badge key={chainId} variant="secondary" className="text-xs">
                            {getChainName(chainId)}
                          </Badge>
                        ))}
                        {endpoint.chain_ids.length > 2 && (
                          <Badge variant="secondary" className="text-xs">
                            +{endpoint.chain_ids.length - 2}
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <StatusBadge status={endpoint.is_active ? 'active' : 'inactive'} size="sm" showIcon={true} />
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">
                      {formatDateTime(endpoint.created_at)}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => router.push(`/dashboard/endpoints/${endpoint.id}`)}
                          title="View Analytics"
                        >
                          <BarChart3 className="w-4 h-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => openEditDialog(endpoint)}
                        >
                          <Edit className="w-4 h-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDelete(endpoint)}
                          className="text-red-600 hover:text-red-700"
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Create Dialog */}
      <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Add Endpoint</DialogTitle>
            <DialogDescription>
              Configure a new webhook endpoint to receive blockchain events
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleCreate}>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="application">Application *</Label>
                <select
                  id="application"
                  className="w-full h-10 px-3 rounded-md border border-input bg-background"
                  value={applicationId}
                  onChange={(e) => setApplicationId(e.target.value)}
                  required
                  data-testid="app-select"
                >
                  <option value="">Select an application</option>
                  {appsData?.applications.map(app => (
                    <option key={app.id} value={app.id}>{app.name}</option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">Name *</Label>
                <Input
                  id="name"
                  placeholder="My Endpoint"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                  data-testid="endpoint-name-input"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="webhookUrl">Webhook URL *</Label>
                <Input
                  id="webhookUrl"
                  type="url"
                  placeholder="https://example.com/webhook"
                  value={webhookUrl}
                  onChange={(e) => setWebhookUrl(e.target.value)}
                  required
                  data-testid="webhook-url-input"
                />
              </div>
              <div className="space-y-2">
                <Label>Chains *</Label>
                <div className="grid grid-cols-2 gap-2">
                  {CHAIN_OPTIONS.map(chain => (
                    <Button
                      key={chain.value}
                      type="button"
                      variant={chainIds.includes(chain.value) ? 'default' : 'outline'}
                      className="justify-start"
                      onClick={() => toggleChain(chain.value)}
                    >
                      {chain.label}
                    </Button>
                  ))}
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="contracts">Contract Addresses (comma-separated)</Label>
                <Input
                  id="contracts"
                  placeholder="0x123..., 0x456..."
                  value={contractAddresses}
                  onChange={(e) => setContractAddresses(e.target.value)}
                />
                <p className="text-xs text-muted-foreground">Leave empty to receive all events</p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="events">Event Signatures (comma-separated)</Label>
                <Input
                  id="events"
                  placeholder="Transfer(address,address,uint256)"
                  value={eventSignatures}
                  onChange={(e) => setEventSignatures(e.target.value)}
                />
                <p className="text-xs text-muted-foreground">Leave empty to receive all events</p>
              </div>
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setIsCreateOpen(false);
                  resetForm();
                }}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending ? 'Creating...' : 'Create Endpoint'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Edit Endpoint</DialogTitle>
            <DialogDescription>
              Update your webhook endpoint configuration
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleEdit}>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="edit-name">Name *</Label>
                <Input
                  id="edit-name"
                  placeholder="My Endpoint"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-webhookUrl">Webhook URL *</Label>
                <Input
                  id="edit-webhookUrl"
                  type="url"
                  placeholder="https://example.com/webhook"
                  value={webhookUrl}
                  onChange={(e) => setWebhookUrl(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label>Chains *</Label>
                <div className="grid grid-cols-2 gap-2">
                  {CHAIN_OPTIONS.map(chain => (
                    <Button
                      key={chain.value}
                      type="button"
                      variant={chainIds.includes(chain.value) ? 'default' : 'outline'}
                      className="justify-start"
                      onClick={() => toggleChain(chain.value)}
                    >
                      {chain.label}
                    </Button>
                  ))}
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-contracts">Contract Addresses (comma-separated)</Label>
                <Input
                  id="edit-contracts"
                  placeholder="0x123..., 0x456..."
                  value={contractAddresses}
                  onChange={(e) => setContractAddresses(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-events">Event Signatures (comma-separated)</Label>
                <Input
                  id="edit-events"
                  placeholder="Transfer(address,address,uint256)"
                  value={eventSignatures}
                  onChange={(e) => setEventSignatures(e.target.value)}
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setIsEditOpen(false);
                  setSelectedEndpoint(null);
                  resetForm();
                }}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending ? 'Updating...' : 'Update Endpoint'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
