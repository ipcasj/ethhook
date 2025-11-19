'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { api } from '@/lib/api-client';
import { User } from '@/lib/types';
import { UserCircle, Mail, Calendar, Save } from 'lucide-react';
import { toast } from 'sonner';
import { formatDateTime } from '@/lib/utils';

export default function SettingsPage() {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [isEditing, setIsEditing] = useState(false);

  // Fetch user profile
  const { data: user, isLoading } = useQuery<User>({
    queryKey: ['user-profile'],
    queryFn: () => api.get<User>('/users/me'),
  });

  // Update profile mutation
  const updateMutation = useMutation({
    mutationFn: (data: { name: string }) =>
      api.put<User>('/users/me', data),
    onSuccess: (updatedUser) => {
      queryClient.setQueryData(['user-profile'], updatedUser);
      setIsEditing(false);
      toast.success('Profile updated successfully!');
    },
    onError: (error) => {
      const errorMessage = error instanceof Error ? error.message : 'Failed to update profile';
      toast.error(errorMessage);
    },
  });

  const handleEdit = () => {
    setName(user?.name || '');
    setIsEditing(true);
  };

  const handleCancel = () => {
    setName('');
    setIsEditing(false);
  };

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      toast.error('Name cannot be empty');
      return;
    }
    updateMutation.mutate({ name: name.trim() });
  };

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
          Settings
        </h1>
        <p className="text-slate-600 mt-1">
          Manage your account settings and preferences
        </p>
      </div>

      {/* Profile Information Card */}
      <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <UserCircle className="w-5 h-5 text-blue-600" />
            Profile Information
          </CardTitle>
          <CardDescription>View and update your personal information</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">Loading profile...</p>
            </div>
          ) : !user ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">Unable to load profile</p>
            </div>
          ) : (
            <form onSubmit={handleSave} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email" className="flex items-center gap-2 text-slate-700">
                  <Mail className="w-4 h-4" />
                  Email Address
                </Label>
                <Input
                  id="email"
                  type="email"
                  value={user.email}
                  disabled
                  className="bg-slate-50 cursor-not-allowed"
                />
                <p className="text-xs text-slate-500">Email address cannot be changed</p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="name" className="flex items-center gap-2 text-slate-700">
                  <UserCircle className="w-4 h-4" />
                  Full Name
                </Label>
                {isEditing ? (
                  <Input
                    id="name"
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Enter your name"
                    autoFocus
                    data-testid="name-input"
                  />
                ) : (
                  <Input
                    id="name"
                    type="text"
                    value={user.name}
                    disabled
                    className="bg-slate-50"
                  />
                )}
              </div>

              <div className="space-y-2">
                <Label className="flex items-center gap-2 text-slate-700">
                  <Calendar className="w-4 h-4" />
                  Account Created
                </Label>
                <Input
                  type="text"
                  value={formatDateTime(user.created_at)}
                  disabled
                  className="bg-slate-50 cursor-not-allowed"
                />
              </div>

              <div className="flex gap-3 pt-4">
                {isEditing ? (
                  <>
                    <Button
                      type="submit"
                      disabled={updateMutation.isPending}
                      className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30"
                    >
                      <Save className="w-4 h-4 mr-2" />
                      {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={handleCancel}
                      disabled={updateMutation.isPending}
                    >
                      Cancel
                    </Button>
                  </>
                ) : (
                  <Button
                    type="button"
                    onClick={handleEdit}
                    className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-indigo-500/30"
                  >
                    Edit Profile
                  </Button>
                )}
              </div>
            </form>
          )}
        </CardContent>
      </Card>

      {/* Account Information Card */}
      <Card className="bg-white/80 backdrop-blur-sm border-0 shadow-lg">
        <CardHeader>
          <CardTitle>Account Information</CardTitle>
          <CardDescription>Additional details about your account</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex justify-between py-2 border-b">
              <span className="text-slate-600">Account ID</span>
              <span className="font-mono text-sm text-slate-900">{user?.id}</span>
            </div>
            <div className="flex justify-between py-2 border-b">
              <span className="text-slate-600">Last Updated</span>
              <span className="text-slate-900">{user?.updated_at ? formatDateTime(user.updated_at) : '-'}</span>
            </div>
            <div className="flex justify-between py-2">
              <span className="text-slate-600">Account Status</span>
              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                Active
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
