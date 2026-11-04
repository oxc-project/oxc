vite = await (
  await import('vite')
).createServer({
  appType
})

const x = (
  await mapLimit((await cache.getAllAccounts()).accounts, 10, async (account) => account)
).flat()
