# wasmasc

AssemblyScript WebAssembly plugin for [Wasmrun](https://github.com/anistark/wasmrun) - compile AssemblyScript projects to WebAssembly with support for both direct `asc` compilation and npm/pnpm/yarn/bun build workflows.

## Installation

### As a Wasmrun Plugin (Recommended)

```sh
wasmrun plugin install wasmasc
```

Wasmrun will automatically detect AssemblyScript projects and prompt you to install this plugin if needed.

### From Source

```sh
cargo install --path . --features cli
```

### From Crates.io

```sh
cargo install wasmasc --features cli
```

## Usage

### Via Wasmrun (Recommended)

```sh
wasmrun run ./my-asc-project
wasmrun compile ./my-asc-project --optimization size
```

### Standalone CLI (Experimental)

```sh
wasmasc compile -p ./my-project -o ./dist
wasmasc compile -p ./my-project --optimization release
wasmasc check-deps
wasmasc can-handle ./my-project
```

## Development

Use `just` commands for common development tasks:

```sh
just dev          # Quick development cycle (format, lint, test, build-cli)
just format       # Format code
just lint         # Lint code
just test         # Run tests
just build        # Build library
just build-cli    # Build with CLI feature
just clean        # Clean build artifacts
just install-cli  # Install locally
```

Run `just --list` to see all available commands.

## Project Structure

Supports standard AssemblyScript project layouts:

```sh
my-asc-project/
├── assembly/
│   ├── index.ts
│   └── main.ts
├── package.json
└── index.ts
```

## Dependencies

### Required
- `asc` - AssemblyScript compiler (`npm install -g asc`)
- `node` - Node.js runtime

### Optional (Package Managers)
The plugin intelligently detects and uses your preferred package manager:
- `npm` - Default Node.js package manager
- `yarn` - Fast, reliable package manager (detected via `yarn.lock`)
- `pnpm` - Efficient, disk space saving package manager (detected via `pnpm-lock.yaml`)
- `bun` - Fast JavaScript runtime with bundled npm alternative (detected via `bun.lockb`)

If none are explicitly locked, the plugin falls back to `npm`.

## Capabilities

- ✅ **Direct ASC Compilation** - Compile `.ts` files to WebAssembly using asc
- ✅ **npm/yarn/pnpm/bun Support** - Automatic package manager detection
- ✅ **Optimization Levels** - Debug, Release, and Size optimizations
- ✅ **Project Auto-detection** - Recognizes AssemblyScript projects automatically
- ✅ **Live Reload** - Supports file watching during development
- ❌ **Web App Packaging** - Not applicable for AssemblyScript

## Examples

### Basic Compilation

```sh
wasmrun compile ./my-asc-project --optimization release
```

### Development with Watching

```sh
wasmrun ./my-asc-project
```

### With Custom Output

```sh
wasmrun compile ./my-asc-project -o ./custom-dist --optimization size
```

## Troubleshooting

### "asc command not found"

Install the AssemblyScript compiler globally:
```sh
npm install -g asc
```

### "No package manager found"

Install at least one of: npm, yarn, pnpm, or bun

### Plugin not detected

Ensure your project has one of:
- `"asc"` or `"@asc"` in `package.json`
- `assembly/index.ts` or `assembly/main.ts`
- `.ts` files in the project

## WASM Plugin Architecture

This plugin is part of the Wasmrun plugin ecosystem. All plugins follow the same FFI-based architecture for dynamic loading and can be updated independently.
This plugin implements the Wasm plugin interface and reads its configuration from `Cargo.toml`. The configuration includes:

- **Extensions**: File extensions the plugin handles
- **Entry Files**: Priority order for entry point detection
- **Capabilities**: What features the plugin supports
- **Dependencies**: Required external tools

#### [License](./LICENSE)
