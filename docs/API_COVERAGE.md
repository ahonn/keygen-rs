# keygen-rs API Coverage Report

Gap analysis of keygen-rs against the [keygen.sh API](https://keygen.sh/docs/api/).

All paths are relative to `/v1/accounts/<account>`.

## Legend

- ✅ Implemented
- ❌ Missing
- ⚠️ Partial (missing filters or parameters)

---

## 1. Licenses

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/licenses` | ✅ |
| GET | `/licenses/<id>` | ✅ |
| PATCH | `/licenses/<id>` | ✅ |
| DELETE | `/licenses/<id>` | ✅ |
| GET | `/licenses` | ⚠️ |

### Actions

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/licenses/<id>/actions/validate` | Validate by ID | ✅ |
| POST | `/licenses/actions/validate-key` | Validate by key | ✅ |
| POST | `/licenses/<id>/actions/suspend` | Suspend | ✅ |
| POST | `/licenses/<id>/actions/reinstate` | Reinstate | ✅ |
| POST | `/licenses/<id>/actions/renew` | Renew | ✅ |
| POST | `/licenses/<id>/actions/revoke` | Revoke | ✅ |
| POST | `/licenses/<id>/actions/check-out` | Check-out (certificate) | ✅ |
| POST | `/licenses/<id>/actions/check-in` | Check-in | ✅ |
| POST | `/licenses/<id>/actions/increment-usage` | Increment usage | ✅ |
| POST | `/licenses/<id>/actions/decrement-usage` | Decrement usage | ✅ |
| POST | `/licenses/<id>/actions/reset-usage` | Reset usage | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/licenses/<id>/tokens` | Generate license token | ✅ |
| POST | `/licenses/<id>/users` | Attach users | ✅ |
| DELETE | `/licenses/<id>/users` | Detach users | ✅ |
| GET | `/licenses/<id>/users` | List users | ✅ |
| POST | `/licenses/<id>/entitlements` | Attach entitlements | ✅ |
| DELETE | `/licenses/<id>/entitlements` | Detach entitlements | ✅ |
| GET | `/licenses/<id>/entitlements` | List entitlements | ✅ |
| PATCH | `/licenses/<id>/policy` | Change policy | ✅ |
| PATCH | `/licenses/<id>/owner` | Change owner | ✅ |
| PATCH | `/licenses/<id>/group` | Change group | ✅ |

### List Filters

| Parameter | Status | Notes |
|-----------|:---:|-------|
| `limit`, `page[size]`, `page[number]` | ✅ | |
| `status` | ✅ | |
| `product` | ✅ | |
| `policy` | ✅ | |
| `owner` | ✅ | |
| `user` | ✅ | |
| `metadata[key]` | ✅ | |
| `group` | ✅ | |
| `machine` | ✅ | |
| `unassigned` / `assigned` | ✅ | |
| `activated` | ✅ | |
| `activations[eq\|gt\|gte\|lt\|lte]` | ✅ | |
| `expires[in\|on\|before\|after]` | ✅ | |
| `expired[in\|on\|before\|after]` | ✅ | |
| `activity[inside\|outside\|before\|after]` | ✅ | |

---

## 2. Users

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/users` | ✅ |
| GET | `/users/<id>` | ✅ |
| PATCH | `/users/<id>` | ✅ |
| DELETE | `/users/<id>` | ✅ |
| GET | `/users` | ✅ |

### Actions

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/users/<id>/actions/update-password` | Update password | ✅ |
| POST | `/users/<id>/actions/reset-password` | Reset password | ✅ |
| POST | `/users/<id>/actions/ban` | Ban user | ✅ |
| POST | `/users/<id>/actions/unban` | Unban user | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/users/<id>/tokens` | Generate user token | ✅ |
| PUT | `/users/<id>/group` | Change/assign group | ✅ |

### Second Factors (2FA)

| Method | Path | Status |
|--------|------|:---:|
| POST | `/users/<user>/second-factors` | ❌ |
| GET | `/users/<user>/second-factors` | ❌ |
| GET | `/users/<user>/second-factors/<id>` | ❌ |
| PATCH | `/users/<user>/second-factors/<id>` | ❌ |
| DELETE | `/users/<user>/second-factors/<id>` | ❌ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `limit`, `page[size]`, `page[number]` | ✅ |
| `status` | ✅ |
| `assigned` | ✅ |
| `product` | ✅ |
| `group` | ✅ |
| `roles[]` | ✅ |
| `metadata[key]` | ✅ |

---

## 3. Machines

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/machines` | ✅ |
| GET | `/machines/<id>` | ✅ |
| PATCH | `/machines/<id>` | ✅ |
| DELETE | `/machines/<id>` | ✅ |
| GET | `/machines` | ⚠️ |

### Actions

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/machines/<id>/actions/check-out` | Offline certificate | ✅ |
| POST | `/machines/<id>/actions/ping` | Heartbeat ping | ✅ |
| POST | `/machines/<id>/actions/reset` | Reset heartbeat | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| PATCH | `/machines/<id>/owner` | Change owner | ✅ |
| PATCH | `/machines/<id>/group` | Change group | ✅ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `limit`, `page[size]`, `page[number]` | ✅ |
| `fingerprint` | ✅ |
| `ip` | ✅ |
| `hostname` | ✅ |
| `product` | ✅ |
| `license` | ✅ |
| `owner` | ✅ |
| `group` | ✅ |
| `metadata[key]` | ✅ |
| `policy` | ✅ |
| `key` | ✅ |

---

## 4. Releases

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/releases` | ✅ |
| GET | `/releases/<id>` | ✅ |
| PATCH | `/releases/<id>` | ✅ |
| DELETE | `/releases/<id>` | ✅ |
| GET | `/releases` | ⚠️ |

### Actions

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/releases/<id>/actions/publish` | Publish | ✅ |
| POST | `/releases/<id>/actions/yank` | Yank | ✅ |

### Special Endpoints

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| GET | `/releases/<id>/upgrade` | Check for upgrade | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| GET | `/releases/<id>/artifacts` | List artifacts | ✅ (via artifact list filter) |
| GET | `/releases/<id>/artifacts/<artifact>` | Download artifact (303) | ✅ |
| POST | `/releases/<id>/constraints` | Attach constraints | ✅ |
| DELETE | `/releases/<id>/constraints` | Detach constraints | ✅ |
| GET | `/releases/<id>/constraints` | List constraints | ✅ |
| PUT | `/releases/<id>/package` | Change package | ✅ |

### List Filters

| Parameter | Status | Notes |
|-----------|:---:|-------|
| `limit`, `page[size]`, `page[number]` | ✅ | |
| `product` | ✅ | |
| `channel` | ✅ | |
| `status` | ✅ | |
| `package` | ✅ | |
| `engine` | ✅ | |
| `entitlements` | ✅ | |

---

## 5. Artifacts

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/artifacts` | ✅ |
| GET | `/artifacts/<id>` | ✅ (download, 303 redirect) |
| PATCH | `/artifacts/<id>` | ✅ |
| DELETE | `/artifacts/<id>` | ✅ (yank) |
| GET | `/artifacts` | ✅ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `limit`, `page[size]`, `page[number]` | ✅ |
| `product` | ✅ |
| `release` | ✅ |
| `channel` | ✅ |
| `filetype` | ✅ |
| `platform` | ✅ |
| `arch` | ✅ |
| `status` | ✅ |

> ✅ Fully covered.

---

## 6. Policies

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/policies` | ✅ |
| GET | `/policies/<id>` | ✅ |
| PATCH | `/policies/<id>` | ✅ |
| DELETE | `/policies/<id>` | ✅ |
| GET | `/policies` | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/policies/<id>/entitlements` | Attach entitlements | ✅ |
| DELETE | `/policies/<id>/entitlements` | Detach entitlements | ✅ |
| GET | `/policies/<id>/entitlements` | List entitlements | ✅ |
| GET | `/policies/<id>/pool` | List pool keys | ✅ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `limit`, `page[size]`, `page[number]` | ✅ |
| `product` | ✅ |

---

## 7. Products

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/products` | ✅ |
| GET | `/products/<id>` | ✅ |
| PATCH | `/products/<id>` | ✅ |
| DELETE | `/products/<id>` | ✅ |
| GET | `/products` | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/products/<id>/tokens` | Generate product token | ✅ |

---

## 8. Tokens

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/tokens` | ✅ (via generate) |
| GET | `/tokens/<id>` | ✅ |
| PUT | `/tokens/<id>` | ✅ (regenerate) |
| DELETE | `/tokens/<id>` | ✅ (revoke) |
| GET | `/tokens` | ⚠️ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `limit`, `page[size]`, `page[number]` | ✅ |
| `bearer[type]` | ✅ |
| `bearer[id]` | ✅ |

---

## 9. Entitlements

| Method | Path | Status |
|--------|------|:---:|
| POST | `/entitlements` | ✅ |
| GET | `/entitlements/<id>` | ✅ |
| PATCH | `/entitlements/<id>` | ✅ |
| DELETE | `/entitlements/<id>` | ✅ |
| GET | `/entitlements` | ✅ |

> ✅ Fully covered.

---

## 10. Components

| Method | Path | Status |
|--------|------|:---:|
| POST | `/components` | ✅ |
| GET | `/components/<id>` | ✅ |
| PATCH | `/components/<id>` | ✅ |
| DELETE | `/components/<id>` | ✅ |
| GET | `/components` | ✅ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `machine`, `license`, `owner`, `user`, `product` | ✅ |

> ✅ Fully covered.

---

## 11. Processes — ❌ Module Missing

| Method | Path | Description |
|--------|------|-------------|
| POST | `/processes` | Create |
| GET | `/processes/<id>` | Retrieve |
| PATCH | `/processes/<id>` | Update |
| DELETE | `/processes/<id>` | Delete |
| GET | `/processes` | List |
| POST | `/processes/<id>/actions/ping` | Heartbeat |

**List Filters:** `machine`, `license`, `owner`, `user`, `product`

---

## 12. Groups

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/groups` | ✅ |
| GET | `/groups/<id>` | ✅ |
| PATCH | `/groups/<id>` | ✅ |
| DELETE | `/groups/<id>` | ✅ |
| GET | `/groups` | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| GET | `/groups/<id>/owners` | List group owners | ✅ |
| GET | `/groups/<id>/users` | List group users | ✅ |
| GET | `/groups/<id>/licenses` | List group licenses | ✅ |
| GET | `/groups/<id>/machines` | List group machines | ✅ |

---

## 13. Environments

### Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/environments` | ✅ |
| GET | `/environments/<id>` | ✅ |
| PATCH | `/environments/<id>` | ✅ |
| DELETE | `/environments/<id>` | ✅ |
| GET | `/environments` | ✅ |

### Relationships

| Method | Path | Description | Status |
|--------|------|-------------|:---:|
| POST | `/environments/<id>/tokens` | Generate token | ✅ |

> ✅ Fully covered.

---

## 14. Packages

| Method | Path | Status |
|--------|------|:---:|
| POST | `/packages` | ✅ |
| GET | `/packages/<id>` | ✅ |
| PATCH | `/packages/<id>` | ✅ |
| DELETE | `/packages/<id>` | ✅ |
| GET | `/packages` | ✅ |

### List Filters

| Parameter | Status |
|-----------|:---:|
| `product` | ✅ |
| `engine` | ✅ |

> ✅ Fully covered.

---

## 15. Webhook Endpoints

| Method | Path | Status |
|--------|------|:---:|
| POST | `/webhook-endpoints` | ✅ |
| GET | `/webhook-endpoints/<id>` | ✅ |
| PATCH | `/webhook-endpoints/<id>` | ✅ |
| DELETE | `/webhook-endpoints/<id>` | ✅ |
| GET | `/webhook-endpoints` | ✅ |

> ✅ Fully covered.

---

## 16. Webhook Events

| Method | Path | Status |
|--------|------|:---:|
| GET | `/webhook-events/<id>` | ✅ |
| DELETE | `/webhook-events/<id>` | ✅ |
| GET | `/webhook-events` | ✅ |
| POST | `/webhook-events/<id>/actions/retry` | ✅ |

> ✅ Fully covered.

---

## 17. Read-Only Distribution Resources

| Resource | list | get | Status |
|----------|:---:|:---:|:---:|
| Platforms | ✅ | ✅ | ✅ |
| Arches | ✅ | ✅ | ✅ |
| Channels | ✅ | ✅ | ✅ |
| Engines | ❌ | ❌ | ❌ |

### Engine Distribution Endpoints — ❌ All Missing

| Method | Path | Description |
|--------|------|-------------|
| GET | `/engines/tauri/<package>` | Tauri auto-update check |
| GET | `/engines/pypi/simple` | PyPI package index |
| GET | `/engines/npm/<package>` | npm package info |
| GET | `/engines/rubygems` | RubyGems index |
| GET | `/engines/oci` | OCI container registry |
| GET | `/engines/raw/<product>/...` | Raw file download |

---

## 18. Other Missing Resources

| Resource | Endpoints | Status |
|----------|-----------|:---:|
| Request Logs | `GET /request-logs`, `GET /request-logs/<id>` | ❌ |
| Event Logs | `GET /event-logs`, `GET /event-logs/<id>` | ❌ |
| Profiles | `GET /me` | ❌ |
| Passwords | `POST /passwords` | ❌ |

---

## Notes

- This report has been updated to reflect the current core crate plus the synced `napi` / `wasm` bindings.
- Remaining gaps below are limited to resources that still do not have modules in this repository, such as processes, engines, request logs, event logs, profiles, passwords, and second factors.
| 8 | `User::generate_token()` | ~20 lines |
| 9 | `ListMachinesOptions` add `policy`, `key` | ~5 lines |
| 10 | `ListTokensOptions` add `bearer_type`, `bearer_id` | ~5 lines |
| 11 | Processes module (CRUD + ping) | ~120 lines |

### P2 — Relationship endpoints & advanced filters (~405 lines Rust)

| # | Change | Est. |
|---|--------|------|
| 12 | License relationships (users, policy, owner, group) | ~80 lines |
| 13 | Machine relationships (owner, group) | ~30 lines |
| 14 | Release constraints (attach, detach, list) | ~50 lines |
| 15 | Policy entitlements + pool | ~50 lines |
| 16 | Groups relationship queries (owners/users/licenses/machines) | ~50 lines |
| 17 | User password actions + group + 2FA | ~100 lines |
| 18 | Product token generation | ~15 lines |
| 19 | License advanced filters (expires, activity, activations) | ~30 lines |

### P3 — Optional / Ops (~210 lines Rust)

| # | Change | Est. |
|---|--------|------|
| 20 | Request Logs (list, get) | ~40 lines |
| 21 | Event Logs (list, get) | ~40 lines |
| 22 | Profiles (`/me`) | ~15 lines |
| 23 | Passwords (forgot) | ~15 lines |
| 24 | Engines distribution endpoints (Tauri/PyPI/npm/etc) | ~100 lines |

### Summary

| Priority | Lines | Description |
|:---:|:---:|------|
| **P0** | ~85 | shipkit blockers |
| **P1** | ~205 | Common operations |
| **P2** | ~405 | Relationships & advanced filters |
| **P3** | ~210 | Ops & distribution |
| **Total** | **~905** | + corresponding wasm/napi bindings |

---

## Fully Covered Modules

These modules require no changes:

- ✅ Artifacts (CRUD + list filters)
- ✅ Entitlements (CRUD)
- ✅ Components (CRUD + list filters)
- ✅ Environments (CRUD + token generation)
- ✅ Packages (CRUD + list filters)
- ✅ Webhook Endpoints (CRUD)
- ✅ Webhook Events (list, get, retry, delete)
- ✅ Platforms (list, get)
- ✅ Arches (list, get)
- ✅ Channels (list, get)
