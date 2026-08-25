# TypeScript Config

Shared TypeScript compiler presets for frontend workspace packages.

## Presets

| File                 | Responsibility                                                                      |
| -------------------- | ----------------------------------------------------------------------------------- |
| `base.json`          | Strict shared compiler defaults for workspace TypeScript projects                   |
| `rsbuild.json`       | Browser/Rsbuild app defaults with DOM libs, source maps, JSON modules, and `noEmit` |
| `react-library.json` | React JSX preset extending `base.json`                                              |

## Usage

Apps extend these configs from their `tsconfig.json` files through `@repo/typescript-config`.

## Verification

Run type checks from the consuming app or from the workspace root:

```sh
pnpm run check
```
