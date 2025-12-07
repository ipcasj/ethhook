'use client';

import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { api } from '@/lib/api-client';
import { User } from '@/lib/types';
import { UserCircle, Mail, Calendar } from 'lucide-react';
import { formatDateTime } from '@/lib/utils';

export default function SettingsPage() {
  // Fetch user profile
  const { data: user, isLoading } = useQuery<User>({
    queryKey: ['user-profile'],
    queryFn: () => api.get<User>('/users/me'),
  });

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold text-slate-900">
          Settings
        </h1>
        <p className="text-slate-600 mt-1">
          Manage your account settings and preferences
        </p>
      </div>

      {/* Profile Information Card */}
      <Card className="bg-white shadow-sm">
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
            <div className="space-y-4">
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
                  Display Name
                </Label>
                <Input
                  id="name"
                  type="text"
                  value={user.email}
                  disabled
                  className="bg-slate-50"
                />
                <p className="text-xs text-slate-500">
                  Display name is set to your email address
                </p>
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

              {/* Edit functionality removed - backend does not support name updates */}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Account Information Card */}
      <Card className="bg-white shadow-sm">
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
