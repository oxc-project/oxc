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

type Span = { start: number; end: number }
export interface SymbolTable {
  spans: Span[]
  names: string[]
  flags: string[]
  scopeIds: number[]
  declarations: number[]
  resolvedReferences: Array<number[]>
  references: Array<{
    span: Span
    name: string
    nodeId: number
    symbolId: number | null
    flag: string
  }>
}

type RenderedSymbol = {
    name: string
    flag: string
    symbolId: number
    nodeId: number
    span: Span
    references: Array<{
        referenceId: number
        span: Span
        name: string
        nodeId: number
        symbolId: number | null
        flag: string
    }>

}

let cacheSymbols: RenderedSymbol[] | null = null

/**
 *
 * @param {SymbolTable} symbols
 * @returns
 */
export const renderSymbols = (symbols: SymbolTable): string => {
  const target = symbols.declarations.reduce(
    (acc, nodeId, index) => {
      acc.push({
        name: symbols.names[index],
        flag: symbols.flags[index],
        symbolId: index,
        nodeId,
        span: symbols.spans[index],
        references: symbols.resolvedReferences[index].map(id => ({
          referenceId: id,
          ...symbols.references[id],
        })),
      })
      return acc
    },
    [] as RenderedSymbol[]
  )
  cacheSymbols = target
  return JSON.stringify(target, null, 2)
}

export const getSymbolAndReferencesSpan = (start: number, end: number): Span[] => {
    if (!cacheSymbols) {
        return [{ start, end }]
    }
    const symbol = cacheSymbols.find(symbol => {
        return symbol.span.start == start && symbol.span.end == end
    })

    if (!symbol) {
        return [{ start, end }]
    }

    return [symbol.span, ...symbol.references.map(reference => reference.span)]
}
