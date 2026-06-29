# syntax

## hello world & base sdk

### sdk.system

```cpp
package main;

using <sdk::system>;

public int main() {
    system.out.println("hello world");
    return 0;
}
```

### sdk.iostream

```cpp
package main;

using <sdk::iostream>;

public int main() {
    iostream.println("Hello world");
    return 0;
}
```

## Memory & Defer

```cpp
package main;

int proccess(int size) {
    ptr<int> data = new int[&size];

    defer delete data;

    data[0] = 100;

    return data[0];
}
```

## record

```cpp
package main;

record Point {
    int x;
    int y;
}

public int main() {
    Point p = Point(10, 20);

    return p.x;
}
```

## switch & do

```cpp
package main;

void check(int x) {
    switch (x) {
        case 1 => {
            //...
        },
        case 2 => {
            // ...
        },
        default => {
            // ...
        }
    }

    int i = 0;
    do (i < 10) {
        i++;
    }
}
```

## slice

```cpp
package main;

int[] get_sub(int[] arr) {
    return arr[0..5];
}
```

## contracts

```cpp
package main;

int divide(int a, int b) {
    contract {
        requires(b != 0, "Division by zero");
        ensures(result >= 0);
    }
    return a / b;
}
```