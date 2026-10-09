const PLACEMENTS = ["leading", "trailing", "dangling"];

/** Attach one native owner's source-ordered comments to its ESTree projection. */
export function attachCommentGroup(node, group, kind, start, end) {
  const projected =
    kind === "Elision"
    || kind === "FormalParameters"
    || kind === "WithClause"
    || kind === "FormalParameter"
    || kind === "CatchParameter"
    || kind === "FormalParameterRest";
  const namedSpecifiers =
    kind === "ImportSpecifiers"
    && node.specifiers.some((specifier) => specifier.type === "ImportSpecifier");
  for (const [comment, placement] of group) {
    const container =
      projected
      || (kind === "ImportSpecifiers"
        && placement === 2
        && (namedSpecifiers || (node.specifiers.length === 0 && comment.end <= node.source.start)));
    const name = PLACEMENTS[placement];
    if (container) {
      comment.container = { kind, placement: name, start, end };
    }
    const buckets = (node.comments ??= { leading: null, trailing: null, dangling: null });
    const bucket = container ? "dangling" : name;
    const comments = (buckets[bucket] ??= []);
    // Flattened owners can merge with a bucket already filled by a descendant.
    // Most appends remain in source order and avoid the insertion search.
    let insertion = comments.length;
    while (insertion > 0 && comments[insertion - 1].start > comment.start) insertion--;
    if (insertion === comments.length) comments.push(comment);
    else comments.splice(insertion, 0, comment);
  }
}
