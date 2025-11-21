(id) =>
  id
    .replace('@', resolve(__dirname, './mods/'))
    .replace('#', resolve(__dirname, '../../'))
    .replace('$', resolve(__dirname, '../foo'))
