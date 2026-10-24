import { apiFetch } from '@/shared/api/client';
import { ENDPOINTS } from '@/shared/api/endpoints';
import type { AuthUser } from '@/shared/types/contracts';
import type { Role } from '@/shared/types/roles';

export interface BackendUser {
  id: string;
  email: string;
  name: string;
  role: string;
  status: string;
  email_verified: boolean;
}

export interface LoginResponse {
  mfa_required: boolean;
  mfa_enrollment_required?: boolean;
  enrollment_token?: string;
  expires_in?: number;
  mfa_token?: string;
  access_token?: string;
  user?: BackendUser;
}

export interface RegisterResponse {
  status: string;
  message: string;
}

export interface MfaEnrollmentResponse {
  otpauth_url: string;
  secret: string;
  recovery_codes: string[];
}

export interface SessionDto {
  id: string;
  current: boolean;
  device_fingerprint?: string;
  user_agent: string;
  ip_address: string;
  created_at: string;
  last_seen_at: string;
  expires_at: string;
}

export function normalizeBackendUser(user: BackendUser): AuthUser {
  const role: Role = user.role === 'ADMIN' || user.role === 'SUPER_ADMIN'
    ? 'SUPER_ADMIN'
    : user.role === 'MODERATOR'
      ? 'MODERATOR'
      : 'CUSTOMER';

  return {
    id: user.id,
    email: user.email,
    name: user.name,
    role,
    status: user.status,
    email_verified: user.email_verified,
  };
}

export function loginFetch(email: string, password: string): Promise<LoginResponse> {
  return apiFetch(ENDPOINTS.LOGIN, {
    method: 'POST',
    body: { email, password },
  });
}

export function registerFetch(
  email: string,
  password: string,
  name?: string
): Promise<RegisterResponse> {
  return apiFetch(ENDPOINTS.REGISTER, {
    method: 'POST',
    body: { email, password, name },
  });
}

export function verifyRegistrationOtp(email: string, code: string) {
  return apiFetch<{ message: string; verified: boolean }>(ENDPOINTS.REGISTER_VERIFY_OTP, {
    method: 'POST',
    body: { email, code },
  });
}

export function resendRegistrationOtp(email: string) {
  return apiFetch<RegisterResponse>(ENDPOINTS.REGISTER_RESEND_OTP, {
    method: 'POST',
    body: { email },
  });
}

export function loginWithTotp(mfaToken: string, code: string): Promise<LoginResponse> {
  return apiFetch(ENDPOINTS.LOGIN_TOTP, {
    method: 'POST',
    body: { mfa_token: mfaToken, code },
  });
}

export function loginWithRecoveryCode(
  mfaToken: string,
  recoveryCode: string
): Promise<LoginResponse> {
  return apiFetch(ENDPOINTS.LOGIN_RECOVERY, {
    method: 'POST',
    body: { mfa_token: mfaToken, recovery_code: recoveryCode },
  });
}

export function enrollMfa(enrollmentToken: string, currentPassword: string) {
  return apiFetch<MfaEnrollmentResponse>(ENDPOINTS.MFA_ENROLL, {
    method: 'POST',
    token: enrollmentToken,
    body: { current_password: currentPassword },
  });
}

export function verifyMfaEnrollment(enrollmentToken: string, code: string) {
  return apiFetch<{ success: boolean }>(ENDPOINTS.MFA_VERIFY, {
    method: 'POST',
    token: enrollmentToken,
    body: { code },
  });
}

export function disableMfa(token: string, currentPassword: string, code: string) {
  return apiFetch<{ success: boolean }>(ENDPOINTS.MFA_DISABLE, {
    method: 'POST',
    token,
    body: { current_password: currentPassword, code },
  });
}

export function requestPasswordReset(email: string) {
  return apiFetch<{ status: string; message: string }>(ENDPOINTS.PASSWORD_FORGOT, {
    method: 'POST',
    body: { email },
  });
}

export function resetPassword(email: string, code: string, newPassword: string) {
  return apiFetch<{ message: string; success: boolean }>(ENDPOINTS.PASSWORD_RESET, {
    method: 'POST',
    body: { email, code, new_password: newPassword },
  });
}

export function changePassword(token: string, currentPassword: string, newPassword: string) {
  return apiFetch<{ message: string; success: boolean; access_token?: string }>(ENDPOINTS.PASSWORD_CHANGE, {
    method: 'POST',
    token,
    body: { current_password: currentPassword, new_password: newPassword },
  });
}

export function refreshTokenFetch(): Promise<{ access_token: string }> {
  return apiFetch(ENDPOINTS.REFRESH, { method: 'POST' });
}

export function logoutFetch(token: string): Promise<{ success: boolean }> {
  return apiFetch(ENDPOINTS.LOGOUT, {
    method: 'POST',
    token,
  });
}

export function fetchCurrentUser(token: string): Promise<BackendUser> {
  return apiFetch(ENDPOINTS.ME, { token });
}

export function fetchSessions(token: string): Promise<SessionDto[]> {
  return apiFetch(ENDPOINTS.SESSIONS, { token });
}

export function revokeSession(token: string, id: string): Promise<{ success: boolean }> {
  return apiFetch(ENDPOINTS.SESSION_BY_ID(id), {
    method: 'DELETE',
    token,
  });
}

export function logoutAllSessions(token: string): Promise<{ success: boolean; sessions_revoked: number }> {
  return apiFetch(ENDPOINTS.LOGOUT_ALL, {
    method: 'POST',
    token,
  });
}
