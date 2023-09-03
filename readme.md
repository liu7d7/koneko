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