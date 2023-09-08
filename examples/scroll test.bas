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
110 v = [36]
120 while 1
125 cls
130 for c = -15 to 0
140 x = 240 + sin((time() + c / 15.) * 4) * 200
150 y = 150 + cos((time() + c / 15.) * 5.2) * 110
155 r = sin(time() + c / 5.) * 10 + 35
160 gosub 100
165 next c
170 refresh
180 loop
1100 for i = 0 to 36
1110 v[i] = {x + sin(rad(i * 10)) * r, y + cos(rad(i * 10)) * r}
1120 next i
1130 poly v c * (-1)
1140 ret
210 v = [36]
220 while 1
225 cls
230 for c = -15 to 0
240 x = 240 + sin((time() + c / 15.) * 4) * 200
250 y = 150 + cos((time() + c / 15.) * 5.2) * 110
255 r = sin(time() + c / 5.) * 10 + 35
260 gosub 100
265 next c
270 refresh
280 loop
2100 for i = 0 to 36
2110 v[i] = {x + sin(rad(i * 10)) * r, y + cos(rad(i * 10)) * r}
2120 next i
2130 poly v c * (-1)
2140 ret