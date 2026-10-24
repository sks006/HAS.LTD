import React, { useState } from 'react';
import {
  registerFetch,
  resendRegistrationOtp,
  verifyRegistrationOtp,
} from '@/slicers/auth_slicer/auth_fetch';
import { ApiError } from '@/shared/api/client';
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
import { Alert, AlertTitle, AlertDescription } from '@/shared/ui/alert';
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/shared/ui/input-otp';
import { UserPlus, Mail, Lock, User, CheckCircle2, ShieldAlert, ArrowRight } from 'lucide-react';

function getErrorMessage(error: unknown): string {
  if (error instanceof ApiError && error.data && typeof error.data === 'object') {
    const body = error.data as { error?: { message?: string }; message?: string };
    return body.error?.message || body.message || error.message;
  }
  return error instanceof Error ? error.message : 'Registration failed. Please check your information.';
}

export const RegisterForm: React.FC<{ onSuccess?: () => void }> = ({ onSuccess }) => {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [code, setCode] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [verificationStep, setVerificationStep] = useState(false);
  const [verified, setVerified] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      if (!verificationStep) {
        const response = await registerFetch(email, password, name || undefined);
        setMessage(response.message || 'Verification code dispatched to your email.');
        setVerificationStep(true);
      } else {
        const response = await verifyRegistrationOtp(email, code);
        if (!response.verified) {
          setError('Email verification could not be completed with this code.');
          return;
        }
        setMessage(response.message || 'Account verified successfully!');
        setVerified(true);
        onSuccess?.();
      }
    } catch (requestError) {
      setError(getErrorMessage(requestError));
    } finally {
      setLoading(false);
    }
  };

  const handleResend = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await resendRegistrationOtp(email);
      setMessage(response.message || 'New verification code dispatched.');
    } catch (requestError) {
      setError(getErrorMessage(requestError));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-md mx-auto border-neutral-200/90 shadow-sm bg-white">
      <CardHeader className="text-center pb-4">
        <div className="w-12 h-12 bg-neutral-900 text-amber-300 rounded-2xl flex items-center justify-center mx-auto mb-2 shadow-xs">
          <UserPlus size={22} />
        </div>
        <CardTitle className="font-serif text-xl font-bold text-neutral-900">
          {verified ? 'Account Activated' : verificationStep ? 'Verify Your Email' : 'Join Ajrah Noor'}
        </CardTitle>
        <CardDescription className="text-xs text-neutral-500">
          {verified
            ? 'Your luxury atelier profile is active. You may now sign in.'
            : verificationStep
            ? `We sent a 6-digit confirmation code to ${email}`
            : 'Register to manage bespoke orders, track shipments, and receive previews'}
        </CardDescription>
      </CardHeader>

      <form onSubmit={handleSubmit}>
        <CardContent className="space-y-4">
          {message && (
            <Alert className="border-emerald-200 bg-emerald-50/70 text-emerald-800">
              <CheckCircle2 className="h-4 w-4 text-emerald-600" />
              <AlertTitle className="text-xs font-bold">Notice</AlertTitle>
              <AlertDescription className="text-xs">{message}</AlertDescription>
            </Alert>
          )}

          {error && (
            <Alert variant="destructive">
              <ShieldAlert className="h-4 w-4" />
              <AlertTitle className="text-xs font-bold">Notice</AlertTitle>
              <AlertDescription className="text-xs">{error}</AlertDescription>
            </Alert>
          )}

          {!verificationStep && (
            <>
              <div className="space-y-1.5">
                <Label htmlFor="regName" className="text-xs font-semibold text-neutral-700">
                  Full Name (Optional)
                </Label>
                <div className="relative">
                  <User className="absolute left-3 top-3 h-4 w-4 text-neutral-400" />
                  <Input
                    id="regName"
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    autoComplete="name"
                    placeholder="Princess Noura"
                    className="pl-9 text-xs"
                  />
                </div>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="regEmail" className="text-xs font-semibold text-neutral-700">
                  Email Address
                </Label>
                <div className="relative">
                  <Mail className="absolute left-3 top-3 h-4 w-4 text-neutral-400" />
                  <Input
                    id="regEmail"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    autoComplete="email"
                    placeholder="you@domain.com"
                    className="pl-9 text-xs"
                  />
                </div>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="regPassword" className="text-xs font-semibold text-neutral-700">
                  Password
                </Label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-4 w-4 text-neutral-400" />
                  <Input
                    id="regPassword"
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    minLength={8}
                    maxLength={128}
                    autoComplete="new-password"
                    placeholder="Minimum 8 characters"
                    className="pl-9 text-xs"
                  />
                </div>
              </div>
            </>
          )}

          {verificationStep && !verified && (
            <div className="space-y-4 text-center">
              <Label className="text-xs font-semibold text-neutral-700 block">
                Enter 6-Digit Email OTP
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
                onClick={handleResend}
                disabled={loading}
                className="text-xs text-amber-800 hover:text-neutral-900 underline block mx-auto disabled:opacity-50"
              >
                Didn't receive code? Resend Email OTP
              </button>
            </div>
          )}
        </CardContent>

        <CardFooter className="flex flex-col space-y-3 pt-2">
          {verified ? (
            <Button
              type="button"
              variant="luxury"
              size="lg"
              className="w-full font-mono text-xs uppercase tracking-wider"
              onClick={onSuccess}
            >
              <span>Continue to Sign In</span>
              <ArrowRight size={14} className="ml-1.5" />
            </Button>
          ) : (
            <Button
              type="submit"
              variant="luxury"
              size="lg"
              className="w-full font-mono text-xs uppercase tracking-wider"
              isLoading={loading}
              disabled={loading}
            >
              {verificationStep ? 'Confirm Verification Code' : 'Create Customer Account'}
            </Button>
          )}
        </CardFooter>
      </form>
    </Card>
  );
};

export default RegisterForm;
