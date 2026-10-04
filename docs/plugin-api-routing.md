# Plugin API routing & common pitfalls

Hard-won notes that have bitten multiple plugins. Read this before wiring an
admin UI to your plugin's `api` endpoints.

## The two-level `/api` prefix (the #1 cause of 404s)

The CMS backend proxies plugin API endpoints at:

```
/api/rest/plugins/<plugin-name>/api/<your-route>
```

- `/api/rest/plugins/<plugin-name>/api/` is a **fixed gateway prefix**.
- `<your-route>` is matched **verbatim** against the routes you declared under
  `api` in `plugin.json`, then forwarded to your plugin binary unchanged.

So the `/api` segment can appear **twice** in the browser URL, depending on how
you named your own routes:

| Manifest `api` route | Your server listens on | Browser must call                               |
| -------------------- | ---------------------- | ----------------------------------------------- |
| `/settings`          | `/settings`            | `/api/rest/plugins/<name>/api/settings`         |
| `/api/settings`      | `/api/settings`        | `/api/rest/plugins/<name>/api/api/settings`     |

Both work — just keep the **manifest route**, your **server route**, and the
**client fetch path** in sync. Pick one convention and stick to it.

## Canonical admin-UI helper

The admin UI is served under `/api/rest/plugins/<name>/ui`, so it can derive the
prefix from its own location:

```js
// base = this plugin's mount point, e.g. /api/rest/plugins/<name>
const base = location.pathname.replace(/\/ui\/?(index\.html)?$/, "");
const api = (p) => base + "/api" + p; // adds the gateway's fixed /api prefix

// Pass your plugin's OWN route, including its /api namespace if you use one:
fetch(api("/settings"));     // plugin route "/settings"      -> .../api/settings
fetch(api("/api/settings")); // plugin route "/api/settings"  -> .../api/api/settings
```

## A 404 here is a path problem, never the method

The proxy forwards **every** HTTP method and the body unchanged (`GET`, `POST`,
`PUT`, `DELETE`, …). Therefore:

- **404** = the path didn't match any declared `api` route. Count the `/api`
  segments; check the manifest route matches your server route.
- **405** = the path matched and reached your plugin, but your server's router
  doesn't handle that method.

Do **not** "fix" a 404 by switching `PUT`→`POST`; the method is never the cause.

## Access control: `allowed_permissions` (not `allowed_roles`)

Each `api` entry in `plugin.json` can restrict who may call it:

```jsonc
"api": [
  { "route": "/settings", "allowed_permissions": ["page.update"] }
]
```

- `allowed_permissions` is an array of **CMS permission ids** (e.g. `page.update`,
  the same ids used across the CMS), with **OR** semantics — the user needs
  **any** one of them.
- If `allowed_permissions` is **omitted or empty**, the route is accessible to
  **every logged-in user** (only authentication is checked, not authorization).

> [!WARNING]
> **`allowed_roles` is deprecated and silently ignored.** It is not part of the
> manifest schema, and the backend never reads it. A route that declares only
> `allowed_roles` has **no authorization check at all** — it is open to every
> logged-in user, regardless of the roles you listed. If you maintain an older
> plugin that still declares `allowed_roles`, migrate it — the role list has no
> effect until you do.

The `ui` endpoint is separate: it is always restricted to users who can manage
plugins, and that is not configurable.

## Calling external APIs from a plugin

Two lessons from real plugins:

- **Surface nested error status, not just the envelope.** Many APIs (e.g.
  onOffice) return a top-level `status` *and* a per-action/nested `status`; the
  specific error (bad field, bad filter) is in the nested one. Logging only the
  envelope gives you blank messages like `Unknown field:` with no field name.
- **An allowlist of fields/columns is all-or-nothing.** One unknown field name
  can make the upstream reject the entire request. Only request documented
  fields, and when you extend the list, verify each name against the API's
  field catalogue.
