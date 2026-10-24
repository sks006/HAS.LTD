import React, { useState } from 'react';
import { useRootStore } from '@/slicers/root_store';
import {
  enrollMfa,
  loginFetch,
  loginWithRecoveryCode,
  loginWithTotp,
  normalizeBackendUser,
  requestPasswordReset,
  resetPassword,
  verifyMfaEnrollment,
} from '@/slicers/auth_slicer/auth_fetch';
import type { LoginResponse, MfaEnrollmentResponse } from '@/slicers/auth_slicer/auth_fetch';
import { ApiError, setApiToken } from '@/shared/api/client';
import type { Role } from '@/shared/types/roles';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from '@/shared/ui/card';
import { Input } from '@/shared/ui/input';
import { Label } from '@/shared/ui/label';
import { Button } from '@/shared/ui/button';
import { Badge } from '@/shared/ui/badge';
import { Separator } from '@/shared/ui/separator';
import { Alert, AlertTitle, AlertDescription } from '@/shared/ui/alert';
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/shared/ui/input-otp';
import {
  Lock,
  Mail,
  KeyRound,
  ShieldAlert,
  ArrowRight,
  ShieldCheck,
  CheckCircle2,
  RefreshCw,
} from 'lucide-react';

type LoginStage = 'login' | 'forgot' | 'reset' | 'mfa' | 'mfa-recovery' | 'enroll' | 'enroll-verify';

function getErrorMessage(error: unknown): string {
  if (error instanceof ApiError && error.data && typeof error.data === 'object') {
    const body = error.data as { error?: { message?: string }; message?: string };
    return body.error?.message || body.message || error.message;
  }
  return error instanceof Error ? error.message : 'The request failed. Please try again.';
}

export const LoginForm: React.FC<{ onSuccess?: (role: Role) => void }> = ({ onSuccess }) => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [code, setCode] = useState('');
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [stage, setStage] = useState<LoginStage>('login');
  const [mfaToken, setMfaToken] = useState('');
  const [enrollmentToken, setEnrollmentToken] = useState('');
  const [enrollment, setEnrollment] = useState<MfaEnrollmentResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const setAuth = useRootStore((state) => state.setAuth);

  const completeLogin = (response: LoginResponse): boolean => {
    if (response.access_token && response.user) {
      const user = normalizeBackendUser(response.user);
      setApiToken(response.access_token);
      setAuth(user, response.access_token);
      onSuccess?.(user.role);
      return true;
    }
    if (response.mfa_required && response.mfa_token) {
      setMfaToken(response.mfa_token);
      setStage('mfa');
      setCode('');
      return true;
    }
    if (response.mfa_enrollment_required && response.enrollment_token) {
      setEnrollmentToken(response.enrollment_token);
      setStage('enroll');
      return true;
    }
    return false;
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      if (stage === 'login') {
        const res = await loginFetch(email, password);
        if (!completeLogin(res)) {
          setError('The server returned an incomplete sign-in response.');
        }
      } else if (stage === 'forgot') {
        const response = await requestPasswordReset(email);
        setStatusMessage(response.message);
        setStage('reset');
      } else if (stage === 'reset') {
        await resetPassword(email, code, newPassword);
        setStatusMessage('Password updated successfully. Please sign in with your new credentials.');
        setStage('login');
        setCode('');
        setNewPassword('');
      } else if (stage === 'mfa') {
        const res = await loginWithTotp(mfaToken, code);
        if (!completeLogin(res)) {
          setError('The server returned an incomplete MFA response.');
        }
      } else if (stage === 'mfa-recovery') {
        const res = await loginWithRecoveryCode(mfaToken, code);
        if (!completeLogin(res)) {
          setError('The server returned an incomplete MFA response.');
        }
      } else if (stage === 'enroll') {
        setEnrollment(await enrollMfa(enrollmentToken, password));
        setStage('enroll-verify');
        setCode('');
      } else {
        await verifyMfaEnrollment(enrollmentToken, code);
        const response = await loginFetch(email, password);
        if (!completeLogin(response)) {
          setError('MFA enabled. Please sign in with your updated credentials.');
          setStage('login');
        }
      }
    } catch (requestError) {
      setError(getErrorMessage(requestError));
    } finally {
      setLoading(false);
    }
  };

  const backToSignIn = () => {
    setStage('login');
    setCode('');
    setError(null);
    setStatusMessage(null);
  };

  return (
    <Card className="w-full max-w-md mx-auto border-neutral-200/90 shadow-sm bg-white">
      <CardHeader className="text-center pb-4">
        <div className="w-12 h-12 bg-neutral-900 text-amber-300 rounded-2xl flex items-center justify-center mx-auto mb-2 shadow-xs">
          <KeyRound size={22} />
        </div>
        <CardTitle className="font-serif text-xl font-bold text-neutral-900">
          {stage === 'login' && 'Atelier Sign In'}
          {stage === 'forgot' && 'Reset Password'}
          {stage === 'reset' && 'Confirm Password Reset'}
          {stage === 'mfa' && 'Two-Factor Verification'}
          {stage === 'mfa-recovery' && 'Emergency Recovery'}
          {stage === 'enroll' && 'Setup Two-Factor Authentication'}
          {stage === 'enroll-verify' && 'Verify Authenticator'}
        </CardTitle>
        <CardDescription className="text-xs text-neutral-500">
          {stage === 'login' && 'Enter your credentials to access your customer or staff account'}
          {stage === 'forgot' && "We'll dispatch a 6-digit verification code to your verified address"}
          {stage === 'reset' && 'Provide your 6-digit code and choose a new secure password'}
          {stage === 'mfa' && 'Enter the 6-digit code from your authenticator application'}
          {stage === 'mfa-recovery' && 'Input one of your 10-character backup recovery codes'}
          {stage === 'enroll' && 'Confirm your current password to initialize authenticator key'}
          {stage === 'enroll-verify' && 'Scan the QR key and confirm the current time-based token'}
        </CardDescription>
      </CardHeader>

      <form onSubmit={handleSubmit}>
        <CardContent className="space-y-4">
          {statusMessage && (
            <Alert className="border-emerald-200 bg-emerald-50/70 text-emerald-800">
              <CheckCircle2 className="h-4 w-4 text-emerald-600" />
              <AlertTitle className="text-xs font-bold">Success</AlertTitle>
              <AlertDescription className="text-xs">{statusMessage}</AlertDescription>
            </Alert>
          )}

          {error && (
            <Alert variant="destructive">
              <ShieldAlert className="h-4 w-4" />
              <AlertTitle className="text-xs font-bold">Authentication Notice</AlertTitle>
              <AlertDescription className="text-xs">{error}</AlertDescription>
            </Alert>
          )}

          {(stage === 'login' || stage === 'forgot') && (
            <>
              <div className="space-y-1.5">
                <Label htmlFor="loginEmail" className="text-xs font-semibold text-neutral-700">
                  Email Address
                </Label>
                <div className="relative">
                  <Mail className="absolute left-3 top-3 h-4 w-4 text-neutral-400" />
                  <Input
                    id="loginEmail"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    autoComplete="username"
                    placeholder="you@domain.com"
                    className="pl-9 text-xs"
                  />
                </div>
              </div>

              {stage === 'login' && (
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="loginPassword" className="text-xs font-semibold text-neutral-700">
                      Password
                    </Label>
                    <button
                      type="button"
                      onClick={() => {
                        setStage('forgot');
                        setError(null);
                        setStatusMessage(null);
                      }}
                      className="text-[11px] text-amber-800 hover:text-neutral-900 transition-colors"
                    >
                      Forgot password?
                    </button>
                  </div>
                  <div className="relative">
                    <Lock className="absolute left-3 top-3 h-4 w-4 text-neutral-400" />
                    <Input
                      id="loginPassword"
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                      autoComplete="current-password"
                      placeholder="••••••••••••"
                      className="pl-9 text-xs"
                    />
                  </div>
                </div>
              )}
            </>
          )}

          {stage === 'reset' && (
            <div className="space-y-4">
              <div className="space-y-2 text-center">
                <Label className="text-xs font-semibold text-neutral-700 block">
                  6-Digit Verification Code
                </Label>
                <div className="flex justify-center">
                  <InputOTP maxLength={6} value={code} onChange={(v) => setCode(v)}>
                    <InputOTPGroup>
                      <InputOTPSlot index={0} />
                      <InputOTPSlot index={1} />
                      <InputOTPSlot index={2} />
                      <InputOTPSlot index={3} />
                      <InputOTPSlot index={4} />
                      <InputOTPSlot index={5} />
                    </InputOTPGroup>
                  </InputOTP>
                </div>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="newPassword" className="text-xs font-semibold text-neutral-700">
                  New Secure Password
                </Label>
                <Input
                  id="newPassword"
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  required
                  minLength={8}
                  maxLength={128}
                  placeholder="Min. 8 characters"
                  className="text-xs"
                />
              </div>
            </div>
          )}

          {stage === 'mfa' && (
            <div className="space-y-4 text-center">
              <Label className="text-xs font-semibold text-neutral-700 block">
                6-Digit Authenticator Code
              </Label>
              <div className="flex justify-center">
                <InputOTP maxLength={6} value={code} onChange={(v) => setCode(v)}>
                  <InputOTPGroup>
                    <InputOTPSlot index={0} />
                    <InputOTPSlot index={1} />
                    <InputOTPSlot index={2} />
                    <InputOTPSlot index={3} />
                    <InputOTPSlot index={4} />
                    <InputOTPSlot index={5} />
                  </InputOTPGroup>
                </InputOTP>
              </div>

              <button
                type="button"
                onClick={() => {
                  setStage('mfa-recovery');
                  setCode('');
                  setError(null);
                }}
                className="text-xs text-amber-800 hover:text-neutral-900 underline block mx-auto"
              >
                Use a 10-character recovery code instead
              </button>
            </div>
          )}

          {stage === 'mfa-recovery' && (
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="recoveryCode" className="text-xs font-semibold text-neutral-700">
                  10-Character Recovery Code
                </Label>
                <Input
                  id="recoveryCode"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  required
                  minLength={10}
                  maxLength={10}
                  placeholder="e.g. A1B2C3D4E5"
                  className="font-mono text-center tracking-widest text-xs"
                />
              </div>

              <button
                type="button"
                onClick={() => {
                  setStage('mfa');
                  setCode('');
                  setError(null);
                }}
                className="text-xs text-amber-800 hover:text-neutral-900 underline block text-center"
              >
                Use standard authenticator app code
              </button>
            </div>
          )}

          {stage === 'enroll' && (
            <div className="space-y-3">
              <p className="text-xs text-neutral-600 leading-relaxed">
                This account role mandates multi-factor authentication. Please verify your current password to issue an enrollment key.
              </p>
              <div className="space-y-1.5">
                <Label htmlFor="enrollPassword" className="text-xs font-semibold text-neutral-700">
                  Current Password
                </Label>
                <Input
                  id="enrollPassword"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  className="text-xs"
                />
              </div>
            </div>
          )}

          {stage === 'enroll-verify' && enrollment && (
            <div className="space-y-4">
              <div className="p-3 bg-neutral-50 rounded-xl border border-neutral-200 text-center space-y-1">
                <span className="text-[10px] uppercase font-bold tracking-widest text-neutral-400 block">
                  Secret Key
                </span>
                <span className="font-mono text-xs font-bold text-neutral-900 select-all break-all">
                  {enrollment.secret}
                </span>
              </div>

              <div className="p-3 bg-amber-50/60 rounded-xl border border-amber-200 text-xs text-amber-900 space-y-1">
                <span className="font-bold block">Save Your 10 Recovery Codes:</span>
                <div className="grid grid-cols-2 gap-1 font-mono text-[10px] pt-1">
                  {enrollment.recovery_codes.map((rc, idx) => (
                    <span key={idx} className="bg-white/80 px-1.5 py-0.5 rounded border border-amber-200/50">
                      {rc}
                    </span>
                  ))}
                </div>
              </div>

              <div className="space-y-2 text-center">
                <Label className="text-xs font-semibold text-neutral-700 block">
                  Enter Generated Code
                </Label>
                <div className="flex justify-center">
                  <InputOTP maxLength={6} value={code} onChange={(v) => setCode(v)}>
                    <InputOTPGroup>
                      <InputOTPSlot index={0} />
                      <InputOTPSlot index={1} />
                      <InputOTPSlot index={2} />
                      <InputOTPSlot index={3} />
                      <InputOTPSlot index={4} />
                      <InputOTPSlot index={5} />
                    </InputOTPGroup>
                  </InputOTP>
                </div>
              </div>
            </div>
          )}
        </CardContent>

        <CardFooter className="flex flex-col space-y-3 pt-2">
          <Button
            type="submit"
            variant="luxury"
            size="lg"
            className="w-full font-mono text-xs uppercase tracking-wider justify-center"
            disabled={loading}
            isLoading={loading}
          >
            {stage === 'login' && 'Sign In to Portal'}
            {stage === 'forgot' && 'Send Reset Code'}
            {stage === 'reset' && 'Update Password'}
            {stage === 'mfa' && 'Verify & Enter'}
            {stage === 'mfa-recovery' && 'Verify Recovery Code'}
            {stage === 'enroll' && 'Generate Setup Key'}
            {stage === 'enroll-verify' && 'Confirm & Activate MFA'}
          </Button>

          {stage !== 'login' && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={backToSignIn}
              className="text-xs text-neutral-500 hover:text-neutral-900"
            >
              Return to Sign In
            </Button>
          )}
        </CardFooter>
      </form>
    </Card>
  );
};

export default LoginForm;
