/**
 * @typedef {Object} SymbolTable
 * @property {Array<{start: number, end: number}>} spans - The spans of the symbols.
 * @property {string[]} names - The names of the symbols.
 * @property {string[]} flags - The flags of the symbols.
 * @property {number[]} scopeIds - The scope IDs of the symbols.
 * @property {number[]} declarations - The declarations of the symbols.
 * @property {Array<number[]>} resolvedReferences - The resolved references of the symbols.
 * @property {Array<{span: {start: number, end: number}, name: string, node_id: number, symbol_id: number|null, flag: string}>} references - The references of the symbols.
 */

/**
 * @type {Array<any>}
 */
let cacheSymbols = null
/**
 * 
 * @param {SymbolTable} symbols 
 * @returns 
 */
export const renderSymbols = (symbols) => {
  const target = []
  symbols.declarations.forEach((nodeId, index) => {
    target.push({
      name: symbols.names[index],
      flag: symbols.flags[index],
      symbolId: index,
      nodeId,
      span: symbols.spans[index],
      references: symbols.resolvedReferences[index].map((id) => symbols.references[id]),
    })
  })
  cacheSymbols = target
  return JSON.stringify(target, null, 2)
}

export const getSymbolAndReferencesSpan = (start, end) => {
  if (!cacheSymbols) {
    return [{ start, end }]
  }
  const symbol = cacheSymbols.find((symbol) => {
    return symbol.span.start == start && symbol.span.end == end
  })

  if (!symbol) {
    return [{ start, end }]
  }

  return [symbol.span, ...symbol.references.map((reference) => reference.span)]
}