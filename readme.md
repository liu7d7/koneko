# koneko

## examples

### hello world

```basic
10 print "hello world!"
20 goto 10
```

```
hello world!
hello world!
...
```

### drawing a triangle

```basic
10 poly {100, 100} {100, 200} {200, 100} "orange"
```

<img src="examples/orange%20triangle.png" width="279" height="273" alt="orange triangle">

### drawing a circle

```basic
10 vertices = [36]
20 radius = 100
30 center_x = 240
40 center_y = 150
50 for i = 0 to 36
60 vertices[i] = {center_x + sin(rad(i * 10)) * radius, center_y + cos(rad(i * 10)) * radius}
70 next i
80 poly vertices "blue"
```

<img src="examples/blue%20circle.png" width="279" height="269" alt="blue circle">

## syntax

### basics (haha‽)

| values                               | example      |
|--------------------------------------|--------------|
| Integer                              | `100`        |
| Float                                | `0.5`        |
| String                               | `"hello!"`   |
| Array                                | `{100, 200}` |
| Null-initialized array of N elements | `[10]`       | 

### built-in statements

These built-in statements can be called in

| name    | syntax                                                                                    | notes                                                                                                                                                                    |
|---------|-------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `for`   | `for [variable] = [begin: int \| float] to [end: int \| float] step [step: int \| float]` | unlike in other basic dialects, `end` is not inclusive. This must be followed up with a `next` statement to loop.                                                        |
| `next`  | `next [variable]`                                                                         | see `for`                                                                                                                                                                |
| `if`    | `if [condition: any] then [true branch] else [false branch]`                              | this must be on a single line. in order to run a block of code conditionally, use `gosub`.                                                                               |
| `while` | `while [condition: any]`                                                                  | this must be followed up with a `loop` statement to loop.                                                                                                                |
| `loop`  | `loop`                                                                                    | see `while`                                                                                                                                                              |
| `gosub` | `gosub [line number: int]`                                                                | jumps to the specified line number, expecting a `ret` statement, which will jump the line after the calling line                                                         |
| `ret`   | `ret`                                                                                     | see `gosub`                                                                                                                                                              |
| `goto`  | `goto [line number: int]`                                                                 | jumps to the specified line number                                                                                                                                       |
| `print` | `print [value: any]`                                                                      | converts `value` to a string and prints it to the screen.                                                                                                                |
| `str`   | `str [value: any]`                                                                        | converts `value` to a string. unlike `print`, `str` will not put delimiters between elements in arrays.                                                                  |
| `int`   | `int [value: string \| float \| int]`                                                     | converts `value` to an integer.                                                                                                                                          |
| `dot`   | `dot [x: int \| float] [y: int \| float] [color: int]`                                    | draws a dot at the specified position.                                                                                                                                   |
| `line`  | `line [x1y1: array<int \| float, 2>] [x2y2: array<int \| float, 2>] [color: int]`         | draws a line from `x1y1` to `x2y2`                                                                                                                                       |
| `poly`  | `poly [vertex_1: array<int \| float, 2>] ... [vertex_n] [color: int]`                     | draws a polygon given any amount of vertices. the vertices are paired up in order to perform the edges, and the last vertex is assumed to connect with the first vertex. |
| `poly`  | `poly [array_of_vertices: array<array<int \| float, 2>, any>] [color: int]`               | same as above, except the vertices are given in an array                                                                                                                 |


