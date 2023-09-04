10 while 1
20 l = int(rnd(3, 6))
30 v = [l]
40 for i = 0 to l
50 v[i] = {rnd(0,400)+40,rnd(0,220)+40}
60 next i
70 poly v int(rnd(1,16))
80 refresh
90 delay 2000
100 cls
110 loop
