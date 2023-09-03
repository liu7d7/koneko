10 v = [36]
20 while 1
25 cls
30 for c = -15 to 0
40 x = 240 + sin((time() + c / 15.) * 4) * 200
50 y = 150 + cos((time() + c / 15.) * 5.2) * 110
55 r = sin(time() + c / 5.) * 10 + 35
60 gosub 100
65 next c
70 refresh
80 loop
100 for i = 0 to 36
110 v[i] = {x + sin(rad(i * 10)) * r, y + cos(rad(i * 10)) * r}
120 next i
130 poly v c * (-1)
140 ret
