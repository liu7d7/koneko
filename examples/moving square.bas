10 x = y = vx = vy = 0
20 cls
30 while key$ = inkey$
40 if key$ == "A" then vx = vx - 1
50 if key$ == "W" then vy = vy - 1
60 if key$ == "D" then vx = vx + 1
70 if key$ == "S" then vy = vy + 1
80 loop
90 if vx & vy then gosub(200)
100 x = x + vx
110 y = y + vy
120 poly {x, y} {x + 10, y} {x + 10, y + 10} {x, y + 10} "orange"
130 delay 16
140 refresh
150 goto 20
200 vx = vx * 0.707
210 vy = vy * 0.707
220 ret
