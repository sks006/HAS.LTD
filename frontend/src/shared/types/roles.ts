export type Role = 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER';

export type Permission =
  | 'inventory:read' | 'inventory:write' | 'inventory:delete'
  | 'product:create' | 'product:update' | 'product:delete'
  | 'order:read' | 'order:status_update' | 'order:refund'
  | 'system:moderator_add' | 'system:moderator_remove' | 'system:metrics_view'
  | 'catalog:read' | 'cart:write' | 'order:create' | 'order:read_own';

export const rolePermissions: Record<Role, Permission[]> = {
  SUPER_ADMIN: [
    'inventory:read', 'inventory:write', 'inventory:delete',
    'product:create', 'product:update', 'product:delete',
    'order:read', 'order:status_update', 'order:refund',
    'system:moderator_add', 'system:moderator_remove', 'system:metrics_view',
  ],
  MODERATOR: [
    'inventory:read', 'product:update', 'order:read', 'order:status_update',
  ],
  CUSTOMER: [
    'catalog:read', 'cart:write', 'order:create', 'order:read_own',
  ],
};

export const can = (role: Role, permission: Permission): boolean =>
  rolePermissions[role].includes(permission);
