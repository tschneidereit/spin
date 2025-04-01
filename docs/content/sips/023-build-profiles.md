title = "SIP 023 - Build profiles"
template = "main"
date = "2025-03-31T12:00:00Z"

---

Summary: Support defining multiple build profiles for components, and running a Spin application in with those profiles.

Owner(s): [till@fermyon.com](mailto:till@fermyon.com)

Created: March 31, 2025

# Background

Building and running individual components or entire applications in configurations other than the default one is currently difficult to do with Spin: one has to manually edit the `[component.name.build]` tables for some or all components. This is laborious and error-prone, and can lead to accidentally committing and deploying debug/testing/profiling builds to production.

# Proposal

This very simple proposal contains three parts:

1. Adding an optional `[component.name.profile.profile-name]` table to use for defining additional build profiles of a component
2. Adding a `-profile [profile-name]` CLI flag and `SPIN_PROFILE` env var to `spin {build, watch, up}` to use specific build profiles of all components (where available, with fallback to the default otherwise)
3. Adding a (repeatable) `-component-profile name=[profile-name]` CLI flag and `SPIN_COMPONENT_PROFILE="name=[profile-name],..."` env var to use specific profiles for specific components

## Example

The following `spin.toml` file shows how these parts are applied:

```toml
# (Source: <https://github.com/fermyon/ai-examples/blob/main/sentiment-analysis-ts/spin.toml>)

spin_manifest_version = 2

[application]
name = "sentiment-analysis"
version = "0.1.0"
authors = ["Caleb Schoepp <caleb.schoepp@fermyon.com>"]
description = "A sentiment analysis API that demonstrates using LLM inference and KV stores together"

[[trigger.http]]
route = "/api/..."
component = "sentiment-analysis"

[component.sentiment-analysis]
source = "target/spin-http-js.wasm"
allowed_outbound_hosts = []
exclude_files = ["**/node_modules"]
key_value_stores = ["default"]
ai_models = ["llama2-chat"]

[component.sentiment-analysis.build]
command = "npm run build"
watch = ["src/**/*", "package.json", "package-lock.json"]

# Debug build of the `sentiment-analysis` component:
[component.sentiment-analysis.profile.debug]
source = "target/spin-http-js.debug.wasm"

[component.sentiment-analysis.profile.debug.build]
command = "npm run build:debug"
watch = ["src/**/*", "package.json", "package-lock.json"]

[[trigger.http]]
route = "/..."
component = "ui"

# The `ui` component doesn't have a debug build, so the release build will always be used.
[component.ui]
source = { url = "<https://github.com/fermyon/spin-fileserver/releases/download/v0.0.1/spin_static_fs.wasm>", digest = "sha256:650376c33a0756b1a52cad7ca670f1126391b79050df0321407da9c741d32375" }
allowed_outbound_hosts = []
files = [{ source = "../sentiment-analysis-assets", destination = "/" }]

[variables]
kv_explorer_user = { required = true }
kv_explorer_password = { required = true }

[[trigger.http]]
component = "kv-explorer"
route = "/internal/kv-explorer/..."

[component.kv-explorer]
source = { url = "<https://github.com/fermyon/spin-kv-explorer/releases/download/v0.10.0/spin-kv-explorer.wasm>", digest = "sha256:65bc286f8315746d1beecd2430e178f539fa487ebf6520099daae09a35dbce1d" }
allowed_outbound_hosts = ["redis://*:*", "mysql://*:*", "postgres://*:*"]
# add or remove stores you want to explore here
key_value_stores = ["default"]

# Uses a pre-built debug version of the component.
[component.kv-explorer.profile.debug]
source = { url = "<https://github.com/fermyon/spin-kv-explorer/releases/download/v0.10.0/spin-kv-explorer.debug.wasm>", digest = ".." }

[component.kv-explorer.variables]
kv_credentials = "{{ kv_explorer_user }}:{{ kv_explorer_password }}"

```

The application defined in this manifest can be run in various configurations:

- `spin up`: uses the release/default builds for everything
- `spin up --profile debug`: uses builds of the profile named `debug` of all components that have them, default builds for the rest
- `spin up --component-profile sentiment-analysis=debug`: uses the debug build of just the `sentiment-analysis` component, default builds for everything else

## Profile-overridable fields

This proposal is focused on build configurations. As such, the fields that are supported in `[component.component-name.profile.profile-name]` tables are limited to:

- `source`
- `command`
- `watch`

This set can be expanded in changes building on this initial support, as adding additional fields should always be backwards-compatible.

## Profiles are all-or-nothing

When running an application with a named profile applied, components that don’t define that profile fall back to the default build configuration. The same isn’t true for individual keys in a profile: if e.g. a profile contains the `source` field but no `command` field, running `spin build` will not try to run the default configuration’s build command.

## Local testing only

At least for now, this proposal is focused entirely on local testing: deployment through plugins is not proposed to gain support for a `--profile` flag.

## Backwards-compatibility

This proposal should be fully backwards-compatible.

## Implementation concerns

I can't see any major implementation concerns: this should hopefully be pretty straightforward.

## Alternatives considered

Instead of adding generalized profiles support, we could add just a hard-coded `debug` profile. This would simplify the syntax a little bit, but at the cost of flexibility.

We could do so by using syntax such as

```toml
[component.sentiment-analysis.debug]
...
```

and changing the `--profile [profile-name]` flag to instead be `--debug`, and `--component-profile sentiment-analysis=debug` be `--debug-component sentiment-analysis`.

I’m not proposing this alternative because the benefits seem much smaller than the downsides.
