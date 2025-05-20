const a = {}

call(a?.b)(a?.c)
call((a?.b))((a?.c))
call(a?.b())(a?.c())
call((a?.b()))((a?.c()))