10 vertices = [36]
20 radius = 100
30 center_x = 240
40 center_y = 150
50 for i = 0 to 36
60 vertices[i] = {center_x + sin(rad(i * 10)) * radius, center_y + cos(rad(i * 10)) * radius}
70 next i
80 poly vertices "blue"