import parser from 'ember-eslint-parser';
import fs from 'fs';

const code = fs.readFileSync('demo-with-issues.gjs', 'utf8');
const result = parser.parseForESLint(code, { filePath: 'demo-with-issues.gjs' });

// Find the class declaration
const classDecl = result.ast.body.find(n => n.type === 'ExportDefaultDeclaration' && n.declaration.type === 'ClassDeclaration');
if (!classDecl) {
  console.log('No class declaration found');
  process.exit(1);
}

const classBody = classDecl.declaration.body.body;
console.log('Class body elements:', classBody.map(e => e.type));

const template = classBody.find(e => e.type === 'GlimmerTemplate');
if (!template) {
  console.log('No template found');
  process.exit(1);
}

console.log('\nTemplate type:', template.type);
console.log('Template keys:', Object.keys(template));

// Find all JavaScript expressions in the template
function findJSExpressions(node, path = []) {
  if (!node || typeof node !== 'object') return [];
  
  const expressions = [];
  
  // Look for GlimmerPathExpression (like "this.count", "this.increment")
  if (node.type === 'GlimmerPathExpression') {
    const pathStr = Array.isArray(node.path) ? node.path.join('.') : (node.path?.original || node.path || 'unknown');
    expressions.push({
      type: 'GlimmerPathExpression',
      path: pathStr,
      original: node,
      span: node.range || node.loc
    });
  }
  
  // Look for GlimmerMustacheStatement which contains expressions
  if (node.type === 'GlimmerMustacheStatement' && node.path) {
    const pathStr = node.path.original || (Array.isArray(node.path.path) ? node.path.path.join('.') : node.path.path || 'unknown');
    expressions.push({
      type: 'GlimmerMustacheStatement',
      path: pathStr,
      original: node.path,
      span: node.range || node.loc
    });
  }
  
  // Look for GlimmerElementModifierStatement (like {{on "click" this.increment}})
  if (node.type === 'GlimmerElementModifierStatement') {
    if (node.path) {
      const pathStr = node.path.original || (Array.isArray(node.path.path) ? node.path.path.join('.') : node.path.path || 'unknown');
      expressions.push({
        type: 'GlimmerElementModifierStatement',
        path: pathStr,
        original: node.path,
        span: node.range || node.loc
      });
    }
    // Also check params
    if (node.params) {
      node.params.forEach(param => {
        if (param.type === 'GlimmerPathExpression') {
          const pathStr = Array.isArray(param.path) ? param.path.join('.') : (param.path?.original || param.path || 'unknown');
          expressions.push({
            type: 'GlimmerPathExpression',
            path: pathStr,
            original: param,
            span: param.range || param.loc
          });
        }
      });
    }
  }
  
  // Recursively search children
  for (const key in node) {
    if (key === 'parent' || key === 'loc' || key === 'range') continue;
    const val = node[key];
    if (Array.isArray(val)) {
      val.forEach(item => expressions.push(...findJSExpressions(item, [...path, key])));
    } else if (val && typeof val === 'object') {
      expressions.push(...findJSExpressions(val, [...path, key]));
    }
  }
  
  return expressions;
}

const jsExpressions = findJSExpressions(template);
console.log('\n\nFound JavaScript expressions in template:');
jsExpressions.forEach(expr => {
  console.log(`  - ${expr.type}: ${expr.path} (at ${expr.span?.[0] || 'unknown'})`);
});

