# Packages

Shared frontend configuration packages live under `packages/`.

| Package                   | Responsibility                                              | README                                           |
| ------------------------- | ----------------------------------------------------------- | ------------------------------------------------ |
| `@repo/rslint-config`     | Shared Rslint config for TypeScript, React, and React Hooks | [rslint-config](rslint-config/README.md)         |
| `@repo/typescript-config` | Shared TypeScript compiler presets                          | [typescript-config](typescript-config/README.md) |

These packages are private workspace dependencies consumed by apps through `workspace:*` references.
