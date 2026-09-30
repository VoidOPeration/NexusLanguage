# Nexus Programming Language

<version : 0.1.0>
<licence : MIT>

* Nexus is a procedural, system-level programming language focused on performance, freedom, and a "C-like" syntax without the historical baggage. It features manual memory management, compile-time execution (comptime), and a hybrid architecture (Native via LLVM & Bytecode VM).

## Features

* Dual Backend: Compile to ultra-fast Native binaries (via LLVM) or portable Bytecode (.nxb).
* Toolchain: 
- nxc: The Compiler.
- nxvm: The Virtual Machine.
- nxp: Package Manager & Build System.
* Memory: Full control. new, delete, defer, and custom Arenas.
* Modern Syntax: C-style syntax with modern features like defer, slice, and records.
* Safety: Contracts, strict typing, and optional checked arithmetic (+?).

## Quick Start
1. Installation

* Build from source using Cargo(Main select):

```bash
cargo build --release
```

The binaries will be in target/release/ (nxc, nxvm, nxp). 

2. Hello World (main.nx) 

```cs
package main;
using <sdk::io>;

public int main() {
    system.out.println("Hello, Nexus!");
    return 0;
}
```
 
3. Compile & Run 

* Native: 
```bash
nxc main.nx -o main
./main
```

* Bytecode: 
```bash
nxc --nxb main.nx
nxvm out.nxb
```
 
## Documentation 

[Language Syntax](./docs/syntax.md)
[SDK Reference](./docs/sdk.md) - non active
[Toolchain Guide](./docs/toolchain.md) - non active
     

# License
MIT