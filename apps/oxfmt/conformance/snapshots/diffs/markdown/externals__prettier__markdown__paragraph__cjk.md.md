# externals/prettier/markdown/paragraph/cjk.md

## Option 2

`````json
{"printWidth":100,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -9,9 +9,11 @@
 This ia an english paragraph with a CJK quote “中文“.
 
 扩展运算符（spread）是三个点（`...`）。
 
-::: warning 注意该网站在国外无法访问，故以下演示无效 :::
+::: warning 注意
+该网站在国外无法访问，故以下演示无效
+:::
 
 IVS 麻󠄁羽󠄀‼️
 
 ⿰あ⿱あ⿲あ⿳あ⿴あ⿵あ⿶あ⿷あ⿸あ⿹あ⿺あ⿻あ

`````

### Actual (oxfmt)

`````md
這是一段很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長的段落

這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！

全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白

This ia an english paragraph with a CJK quote "中文".

This ia an english paragraph with a CJK quote “中文“.

扩展运算符（spread）是三个点（`...`）。

::: warning 注意
该网站在国外无法访问，故以下演示无效
:::

IVS 麻󠄁羽󠄀‼️

⿰あ⿱あ⿲あ⿳あ⿴あ⿵あ⿶あ⿷あ⿸あ⿹あ⿺あ⿻あ

1.a 2.b 3.中

`````

### Expected (prettier)

`````md
這是一段很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長很長的段落

這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！這是一個English混合著中文的一段Paragraph！

全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白全　　形　空白

This ia an english paragraph with a CJK quote "中文".

This ia an english paragraph with a CJK quote “中文“.

扩展运算符（spread）是三个点（`...`）。

::: warning 注意该网站在国外无法访问，故以下演示无效 :::

IVS 麻󠄁羽󠄀‼️

⿰あ⿱あ⿲あ⿳あ⿴あ⿵あ⿶あ⿷あ⿸あ⿹あ⿺あ⿻あ

1.a 2.b 3.中

`````
