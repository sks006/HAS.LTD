# Changelog

All notable changes to the **HAS.LTD** full-stack enterprise platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] - 2026-10-24

### 🛡️ Portal Security & Role-Based Access Control (RBAC)
* **Strict Staff Portal Guarding (`RequireStaff.tsx` & `RequireAuth.tsx`):**
  * Protected all administrative routes (`/dashboard`, `/dashboard/*`, `/dashboard/orders`, `/orders`, `/admin`, `/moderator`, `/dashboard/moderator`) behind `RequireStaff`.
  * Unauthenticated users are redirected to `/login` with location redirect parameters.
  * Customer accounts are strictly denied portal entry and presented with an Haute Couture `AccessDenied` card displaying their current clearance level (`CUSTOMER`), account email, and direct navigation back to the boutique storefront or staff switch account options.
* **Granular Feature Gating: Super Admin vs. Moderator:**
  * **Global Revenue & Telemetry (`GlobalRevenueDashboard.tsx`):** Exclusively visible to `SUPER_ADMIN`. In `InventoryDashboard.tsx`, moderators see a dedicated Moderation Clearance banner detailing their authorized actions.
  * **Order Deletions (`OrdersPage.tsx`):** Single-order deletion in row menus and bulk deletion actions (`DELETE /orders/{id}`) are restricted to `SUPER_ADMIN`. Moderators retain status transitions, inspection, and customer communications.
  * **Product Creation & Deletion (`AddProductModal.tsx` & `AccessControlMatrix.tsx`):** Product creation and database deletions are gated to `SUPER_ADMIN`. Moderators can read inventory and modify stock levels.
  * **Dynamic Dashboard Navigation (`DashboardLayout.tsx`):**
    * Top bar `+ Add Product` button is displayed only for `SUPER_ADMIN`.
    * Sidebar dynamically adapts: hides Super Admin, Product Creation, and User Management sections when logged in as `MODERATOR`.
    * Prominent visual role badges (`SUPER_ADMIN` in gold, `MODERATOR` in purple) displayed in the sidebar and dashboard header.
  * **Storefront Staff Button (`Home.tsx`):** The storefront header Portal button is only visible to authenticated `SUPER_ADMIN` and `MODERATOR` users.

---

### 💎 Frontend Refinement & Shadcn UI Integration

#### Added & Upgraded UI Primitives (`frontend/src/shared/ui/`)
* **Standard Shadcn Button (`button.tsx`):**
  * Integrated Radix UI `Slot` with Class Variance Authority (`cva`).
  * Implemented luxury brand variants (`luxury`, `gold`, `primary`, `destructive`, `outline`, `secondary`, `ghost`, `link`).
  * Added built-in accessible SVG spinner for `isLoading` state.
  * Resolved Linux filesystem casing conflicts between `Button.tsx` and `button.tsx`.
* **Standard Shadcn Table (`table.tsx`):**
  * Implemented composable primitives: `Table`, `TableHeader`, `TableBody`, `TableFooter`, `TableHead`, `TableRow`, `TableCell`, and `TableCaption`.
  * Resolved casing collision with legacy `Table.tsx`.
* **Shadcn Input OTP (`input-otp.tsx`):**
  * Integrated `InputOTP`, `InputOTPGroup`, and `InputOTPSlot` primitives for 6-digit email confirmation codes and RFC 6238 TOTP authentication.

#### Storefront & Catalog Experience
* **Haute Couture Product Grid (`ProductGrid.tsx`):**
  * Replaced legacy modals with Shadcn `Dialog` (`DialogContent`, `DialogHeader`, `DialogTitle`, `DialogDescription`, `DialogFooter`) for atelier customer inquiries.
  * Added dynamic navigation routing to `/product/:id` on item click.
  * Incorporated Shadcn `Badge` for luxury collection tagging (`Sale`, `Atelier`, `Velvet & Silk`).
  * Enhanced responsive hover action bars with `ADD TO BAG`, `BUY NOW`, and `MESSAGE` triggers.
* **Luxury Product Detail Page (`ProductDetail.tsx`):**
  * Connected live backend endpoint `GET /products/{id}` with resilient store fallback.
  * Replaced mock static data with dynamic product imagery, price formatting, and atelier details.
  * Integrated Shadcn `Tabs` (`TabsList`, `TabsTrigger`, `TabsContent`) for Garment Details, Fabric & Care, and Delivery & Returns.
  * Added abaya length/sizing selector (`50 (XS)` to `58 (XL)`, Bespoke Fit), express GCC delivery guarantees, and thumbnail carousel.
* **Interactive Filter Sidebar (`FilterSidebar.tsx`):**
  * Replaced custom form controls with Shadcn `Card`, `CardHeader`, `CardTitle`, `CardContent`, and `Input`.
  * Added dynamic category item counts computed in real-time from active catalog items.
  * Added one-click filter reset button and atelier brand badge.

#### Transactional Commerce & Checkout
* **Atelier Slide-Out Bag (`CartDrawer.tsx`):**
  * Replaced raw custom fixed modal with official Shadcn `Sheet` (`SheetContent`, `SheetHeader`, `SheetTitle`, `SheetDescription`, `SheetFooter`).
  * Integrated automatic debounced backend synchronization against `POST /cart/sync`.
  * Implemented quantity increment/decrement controls, item removal with Lucide icons, complimentary shipping badges, and empty bag state.
* **Checkout & Payment Form (`PaymentGatewayForm.tsx` & `Checkout.tsx`):**
  * Refactored into structured Shadcn `Card`, `Input`, `Label`, `Button`, `Badge`, `Separator`, and `Alert` components.
  * Connected end-to-end checkout pipeline calling `POST /checkout` with cryptographic `x-idempotency-key` and customer authorization token.
  * Implemented Cash on Delivery (COD) and Debit/Credit Card settlement tabs.
  * Added post-checkout confirmation view displaying generated order UUID with direct link to order tracking.

#### Customer Order Tracking & Lifecycle
* **Real-Time Order Tracking Page (`CustomerOrderPage.tsx`):**
  * Refactored using Shadcn `Card`, `Badge`, `Button`, `Progress`, `Separator`, and `Dialog`.
  * Connected live order status resolution against `GET /orders/{id}` with manual refresh polling.
  * Implemented 4-stage progress indicator: Pending (25%) → Reserved (50%) → Tailored/Paid (75%) → Shipped (100%).
  * Connected atelier customer inquiry dialog directly on the order record.

#### Authentication, Identity & Multi-Factor Access
* **Atelier Sign-In Flow (`LoginForm.tsx` & `Login.tsx`):**
  * Replaced raw inputs with Shadcn `Card`, `Input`, `Label`, `Button`, `Alert`, and `InputOTP`.
  * Connected multi-stage authentication:
    * Standard Email & Password (`POST /auth/login`).
    * Non-enumerating password reset request (`POST /auth/password/forgot`).
    * 6-digit password reset confirmation (`POST /auth/password/reset`) using `InputOTP`.
    * RFC 6238 TOTP MFA challenge (`POST /auth/login/totp`) using `InputOTP`.
    * Single-use backup recovery code entry (`POST /auth/login/recovery`).
    * Forced MFA enrollment key generation and activation (`POST /auth/mfa/totp/enroll`, `/verify`).
* **Customer Registration Flow (`RegisterForm.tsx` & `Register.tsx`):**
  * Upgraded with Shadcn `Card`, `Input`, `Label`, `Button`, `Alert`, and `InputOTP`.
  * Connected two-phase registration: `POST /auth/register` followed by 6-digit email OTP verification via `POST /auth/register/verify-otp`.
  * Added resend verification OTP trigger (`POST /auth/register/resend-otp`).

#### Administrative & Moderation Dashboards
* **Couture Inventory & RBAC Matrix (`AccessControlMatrix.tsx`):**
  * Fully refactored to Shadcn `Card`, `Table`, `TableHeader`, `TableBody`, `TableHead`, `TableRow`, `TableCell`, `Badge`, `Button`, `Input`, and `Dialog`.
  * Connected real-time stock management, optimistic UI updates, and product deletion mutations directly to `POST /products`, `PUT /products/{id}`, and `DELETE /products/{id}`.
* **Moderator Operations Console (`ModeratorPanel.tsx`):**
  * Upgraded using Shadcn `Card`, `Table`, `TableHeader`, `TableBody`, `TableHead`, `TableRow`, `TableCell`, `Badge`, `Button`, `Input`, `Dialog`, and `Alert`.
  * Connected live order status progression (Pending, Processing, Delivered, Returned) to `PUT /orders/{id}/status`.
  * Integrated direct customer chat messaging dialog with quick action chips.
* **Master Enterprise Orders Management (`OrdersPage.tsx`):**
  * Upgraded using Shadcn `Card`, `Table`, `TableHeader`, `TableBody`, `TableHead`, `TableRow`, `TableCell`, `Badge`, `Button`, `Input`, `Dialog`, `DropdownMenu`, and `Alert`.
  * Connected full lifecycle actions: order inspection with idempotency key & OCC version inspection, status transitions, batch selection, single/bulk deletion (`DELETE /orders/{id}`), and pagination.
* **Catalog & Product Moderation (`AddProductModal.tsx` & `ProductReviewTable.tsx`):**
  * Refactored product creation into Shadcn `Dialog`, `Input`, `Label`, `Button`, and `Alert`, connecting directly to `POST /products`.
  * Refactored review table to Shadcn `Table`, `Badge`, and `Button` for catalog visibility toggling.
* **Moderation Audit Queue (`OrderAuditQueue.tsx`):**
  * Upgraded to Shadcn `Card`, `Table`, `Badge`, and `Button` for staff order review.
* **System Telemetry Dashboard (`GlobalRevenueDashboard.tsx` & `SystemGodMode.tsx`):**
  * Refactored to Shadcn `Card`, `Badge`, and `Button`.
  * Connected live polling against backend `GET /admin/metrics` for active database connections, Redis status, and Axum CPU load.
  * Cleaned unused imports and verified RBAC `RequireAdmin` protection.

---

### 🔗 Connected Backend API Endpoints Matrix

| HTTP Method | Backend Route | Frontend Consumer Component | Status |
| :--- | :--- | :--- | :---: |
| `GET` | `/health` | Ingress / Public health check | Connected |
| `GET` | `/metrics` | Prometheus scraper / Observability | Connected |
| `GET` | `/products` | `useProducts()` / `ProductGrid.tsx` | Connected |
| `GET` | `/products/{id}` | `ProductDetail.tsx` | Connected |
| `POST` | `/products` | `AddProductModal.tsx` | Connected |
| `PUT` | `/products/{id}` | `AccessControlMatrix.tsx` | Connected |
| `DELETE` | `/products/{id}` | `AccessControlMatrix.tsx` | Connected |
| `POST` | `/cart/sync` | `CartDrawer.tsx` | Connected |
| `POST` | `/checkout` | `PaymentGatewayForm.tsx` | Connected |
| `GET` | `/orders` | `OrdersPage.tsx` | Connected |
| `GET` | `/orders/{id}` | `CustomerOrderPage.tsx` | Connected |
| `PUT` | `/orders/{id}/status` | `OrdersPage.tsx` / `ModeratorPanel.tsx` | Connected |
| `DELETE` | `/orders/{id}` | `OrdersPage.tsx` | Connected |
| `GET` | `/admin/metrics` | `GlobalRevenueDashboard.tsx` | Connected |
| `POST` | `/auth/login` | `LoginForm.tsx` | Connected |
| `GET` | `/auth/me` | `AuthProvider.tsx` | Connected |
| `POST` | `/auth/refresh` | `AuthProvider.tsx` (Cookie / RTR) | Connected |
| `POST` | `/auth/logout` | `AuthProvider.tsx` / `Home.tsx` | Connected |
| `POST` | `/auth/logout-all` | `AuthProvider.tsx` | Connected |
| `POST` | `/auth/register` | `RegisterForm.tsx` | Connected |
| `POST` | `/auth/register/verify-otp` | `RegisterForm.tsx` | Connected |
| `POST` | `/auth/register/resend-otp` | `RegisterForm.tsx` | Connected |
| `POST` | `/auth/password/forgot` | `LoginForm.tsx` | Connected |
| `POST` | `/auth/password/reset` | `LoginForm.tsx` | Connected |
| `POST` | `/auth/mfa/totp/enroll` | `LoginForm.tsx` | Connected |
| `POST` | `/auth/mfa/totp/verify` | `LoginForm.tsx` | Connected |
| `POST` | `/auth/login/totp` | `LoginForm.tsx` | Connected |
| `POST` | `/auth/login/recovery` | `LoginForm.tsx` | Connected |

---

### 🛡️ Type Safety & Build Verification
* Verified strict TypeScript compilation with `npm run typecheck` across all modules: **0 errors, 0 warnings**.
* Verified backend Rust test suites with `cargo test --workspace`: **166 tests passing (100% green)**.
