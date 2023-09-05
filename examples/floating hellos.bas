5 a$ = "hello!"
10 while 1
15 cls
20 for i = -15 to 1
30 o = i * 0.15
40 c = i * (-1)
50 gosub 100
60 gosub 200
70 next i
80 refresh
90 loop
100 text a$ sin(time() + o) * 226 + 226 cos((time() + o) * 1.3) * 145 + 145 c
110 ret
200 text a$ cos(time() + o) * 226 + 226 sin((time() + o) * 1.3) * 145 + 145 c
210 ret
