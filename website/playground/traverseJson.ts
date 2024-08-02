import { SyntaxNode } from "@lezer/common";

const typeFilter = ["JsonText", "Object", "Property", "Array"];

export function findMostInnerNodeForPosition(
  node: SyntaxNode,
  offset: number,
  source: string
): SyntaxNode | undefined {
	if (!typeFilter.includes(node.name)) {
		return;
	}
	let targetNode: SyntaxNode | undefined;
	if (node.name === "Object") {
		let span = getSpanOfNode(node, source);
		if (Object.keys(span).length > 0) {
			let { start, end } = span;
			if (start <= offset && end >= offset) {
				targetNode = node;
			} else {
				return targetNode;
			}
		}
	}
	let curChild = node.firstChild;
	while (curChild) {
		let node = findMostInnerNodeForPosition(curChild, offset, source);
		if (node?.from) {
			targetNode = node;
		}
		curChild = curChild.nextSibling;
	}
	return targetNode;
}

/**
 * @param {import('@lezer/common').SyntaxNode} node
 * @param {string} source
 * */
function getSpanOfNode(node: SyntaxNode, source: string) {
	let span = {} as Record<string, number>;

	for (let child = node.firstChild; !!child; child = child.nextSibling) {
		if (child.name === "Property" && child.firstChild?.name === "PropertyName") {
			let { from, to } = child.firstChild;
			let name = source.slice(from + 1, to - 1);
			if (["start", "end"].includes(name)) {
				let value = child.firstChild.nextSibling;
				if (value) {
					span[name] = +source.slice(value.from, value.to);
				}
			}
		}
	}

	return span;
}
