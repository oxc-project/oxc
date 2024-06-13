type Span = { start: number; end: number }
export interface SymbolTable {
  /**
   * The spans of the symbols.
   */
  spans: Span[]
  /**
   * The names of the symbols.
   */
  names: string[]
  /**
   * The flags of the symbols.
   */
  flags: string[]
  /**
   * The scope IDs of the symbols.
   */
  scopeIds: number[]
  /**
   * The declarations of the symbols.
   */
  declarations: number[]
  /**
   * The resolved references of the symbols.
   */
  resolvedReferences: Array<number[]>
  /**
   * The references of the symbols.
   */
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
