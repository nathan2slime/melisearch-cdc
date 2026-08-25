# Rslint Config

Shared Rslint configuration package published inside the workspace as `@repo/rslint-config`.

## Responsibility

Exports one config from `index.ts` that combines:

| Config                                 | Purpose           |
| -------------------------------------- | ----------------- |
| `js.configs.recommended`               | JavaScript rules  |
| `ts.configs.recommended`               | TypeScript rules  |
| `reactPlugin.configs.recommended`      | React rules       |
| `reactHooksPlugin.configs.recommended` | React Hooks rules |

It also ignores `dist/**` and `node_modules/**`.

## Usage

Apps import the package from their Rslint config and depend on it with `workspace:*`.

## Verification

Run lint from the consuming app or from the workspace root:

```sh
pnpm run lint
```
