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
```

# board

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x228 -> 0x38 -> 0x0 (column each u64 array to 10 times) -> 0x10 (row each i32 to 40 times)
```

# hold

## 1p

```

```

# current piece

## 1p

maybe ?

```
"PuyoPuyoTetris2.exe"+01F17FE0 -> C
```

# next

## 1p

```
"PuyoPuyoTetris2.exe"+01F260D0 -> 0x60 -> 0x98 -> 0x168 (each u32 and concat FFFF to 5times)  
```