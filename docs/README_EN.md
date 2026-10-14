# Nexus Programming Language

<version : 0.1.0>
<licence : MIT>

* **Nexus** is a procedural-oriatation, ***Resource-orientation***, **Object-oriantation(data-oriantation)**, imperative,  low-level programming language focused on performance, freedom, 'n', a "C-like" syntax without the historical baggage. It features manual memory management, compile-time execution (comptime), and Native compilation via LLVM.

## Features

* One Backend: Compile to ultra-fast Native binaries (via LLVM).
* Toolchain: 
- nxc: The Compiler.
- anvil: The build system.
- forge: Package Manager.
* Memory: Full control. *new*, *delete*, *defer*, and custom Arenas.
* Modern Syntax: C-style syntax with modern features like *defer*, *slice*, and *records*.
* Safety: Contracts, strict static typing.

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
using < sdk::system >;

public int main() {
    system.println("Hello, Nexus!");
    return 0;
}
```
 
3. Compile & Run 

* Native: 
```bash
nxc main.nx -o main
./main
```

## Documentation 

[Language Syntax](./docs/syntax.md)
[SDK Reference](./docs/sdk.md) - non active
[Toolchain Guide](./docs/toolchain.md) - non active
     

# License
MIT
