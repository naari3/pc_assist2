# color type

```
-1 empty
0 S green
1 Z red
2 J blue
3 L orange
4 T purple
5 O yellow
6 I cyan
7 . dot
8 __ ghost
```

# board

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x1CB8 -> 0x18 -> 0x0,0x8,0x10...0x48 (column each u64 array to 10 times) -> 0x0,0x4,0x8...0x9C (row each i32 to 40 times)
```

# hold

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x1CC8 (if hold is none, there is 0x0) -> 0x8
```

# current piece

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x1CC0 (if current piece is none, there is 0x0) -> 0x8
```

# next

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x60 -> 0x98 -> 0x168 (each u32 and concat FFFF to 5times)  
```
