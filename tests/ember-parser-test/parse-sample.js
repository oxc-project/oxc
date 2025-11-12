import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function parseFile(filename) {
  console.log(`\n${'='.repeat(80)}`);
  console.log(`Parsing: ${filename}`);
  console.log('='.repeat(80));

  const filePath = path.join(__dirname, filename);
  const sourceCode = fs.readFileSync(filePath, 'utf8');

  console.log('\n--- Source Code ---');
  console.log(sourceCode);

  try {
    // Import the parser dynamically
    const parserModule = await import('ember-eslint-parser');
    const parser = parserModule.default || parserModule;

    console.log('\n--- Parser Info ---');
    console.log('Parser object keys:', Object.keys(parser));

    // Check which method is available
    if (typeof parser.parseForESLint === 'function') {
      console.log('Using parseForESLint()');

      const result = parser.parseForESLint(sourceCode, {
        ecmaVersion: 2022,
        sourceType: 'module',
        filePath: filePath,
      });

      console.log('\n--- parseForESLint Result Keys ---');
      console.log(Object.keys(result));

      // Save full AST to file (removing circular references)
      const astOutputPath = path.join(__dirname, `${filename}.ast.json`);
      const seen = new WeakSet();
      const circularReplacer = (key, value) => {
        if (key === 'parent') return undefined; // Skip parent references
        if (typeof value === 'object' && value !== null) {
          if (seen.has(value)) {
            return '[Circular]';
          }
          seen.add(value);
        }
        return value;
      };

      fs.writeFileSync(
        astOutputPath,
        JSON.stringify(result.ast, circularReplacer, 2),
        'utf8'
      );
      console.log(`\nFull AST saved to: ${astOutputPath}`);

      // Show AST summary
      console.log('\n--- AST Summary ---');
      console.log('Type:', result.ast.type);
      console.log('Body length:', result.ast.body?.length);
      console.log('Source type:', result.ast.sourceType);
      console.log('Has comments:', result.ast.comments?.length || 0);
      console.log('Has tokens:', result.ast.tokens?.length || 0);

      // Show body nodes
      if (result.ast.body) {
        console.log('\n--- Body Nodes ---');
        result.ast.body.forEach((node, i) => {
          console.log(`[${i}] ${node.type}`);
          if (node.type === 'ExportDefaultDeclaration' && node.declaration) {
            console.log(`    ↳ exports: ${node.declaration.type}`);
          }
          if (node.type === 'ImportDeclaration') {
            console.log(`    ↳ from: ${node.source?.value}`);
          }
        });
      }

      // Look for template-related nodes
      console.log('\n--- Searching for Template Nodes ---');
      searchForTemplateNodes(result.ast);

      // Show services if available
      if (result.services) {
        console.log('\n--- Parser Services ---');
        console.log('Services keys:', Object.keys(result.services));
      }

      // Show scopeManager if available
      if (result.scopeManager) {
        console.log('\n--- Scope Manager ---');
        console.log('Has scopeManager:', true);
        console.log('Scope count:', result.scopeManager.scopes?.length || 0);
      }

      // Show visitorKeys if available
      if (result.visitorKeys) {
        console.log('\n--- Visitor Keys ---');
        console.log('Custom visitor keys:', Object.keys(result.visitorKeys).length);
      }

    } else if (typeof parser.parse === 'function') {
      console.log('Using parse()');

      const ast = parser.parse(sourceCode, {
        ecmaVersion: 2022,
        sourceType: 'module',
        filePath: filePath,
      });

      const astOutputPath = path.join(__dirname, `${filename}.ast.json`);
      fs.writeFileSync(
        astOutputPath,
        JSON.stringify(ast, null, 2),
        'utf8'
      );
      console.log(`\nFull AST saved to: ${astOutputPath}`);

    } else {
      console.error('Parser does not have parse() or parseForESLint() method!');
      console.log('Available methods:', Object.keys(parser));
    }

  } catch (error) {
    console.error('\n--- Error ---');
    console.error('Error parsing file:', error.message);
    console.error(error.stack);
  }
}

function searchForTemplateNodes(node, depth = 0, path = 'root') {
  if (!node || typeof node !== 'object') return;

  const indent = '  '.repeat(depth);

  // Check for template-related properties
  if (node.type && (
    node.type.includes('Template') ||
    node.type.includes('Glimmer') ||
    node.type.includes('Content')
  )) {
    console.log(`${indent}Found: ${path} -> ${node.type}`);
  }

  // Check for special property names
  const templateProps = ['template', 'templates', 'contentDocumentFragment'];
  for (const prop of templateProps) {
    if (node[prop]) {
      console.log(`${indent}Found property: ${path}.${prop}`);
      if (typeof node[prop] === 'object') {
        console.log(`${indent}  Type: ${node[prop].type || 'object'}`);
      }
    }
  }

  // Recurse into object properties
  if (Array.isArray(node)) {
    node.forEach((item, i) => {
      searchForTemplateNodes(item, depth + 1, `${path}[${i}]`);
    });
  } else {
    for (const [key, value] of Object.entries(node)) {
      if (key !== 'parent' && key !== 'loc' && key !== 'range') {
        searchForTemplateNodes(value, depth + 1, `${path}.${key}`);
      }
    }
  }
}

// Parse both files
async function main() {
  await parseFile('sample.gjs');
  await parseFile('sample.gts');

  console.log('\n' + '='.repeat(80));
  console.log('Parsing complete! Check the .ast.json files for full output.');
  console.log('='.repeat(80));
}

main().catch(console.error);
