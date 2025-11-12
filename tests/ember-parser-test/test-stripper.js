import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
  stripCustomNodes,
  collectCustomNodeTypes,
  validateESTreeAST,
} from './strip-custom-nodes.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function testStripper(filename) {
  console.log(`\n${'='.repeat(80)}`);
  console.log(`Testing stripper on: ${filename}`);
  console.log('='.repeat(80));

  // Load the parsed AST
  const astPath = path.join(__dirname, `${filename}.ast.json`);
  const ast = JSON.parse(fs.readFileSync(astPath, 'utf8'));

  console.log('\n--- Original AST ---');
  console.log('Type:', ast.type);
  console.log('Body nodes:', ast.body?.length);

  // Collect custom node types
  const customTypes = collectCustomNodeTypes(ast);
  console.log('\n--- Custom Node Types Found ---');
  console.log('Count:', customTypes.size);
  if (customTypes.size > 0) {
    console.log('Types:', Array.from(customTypes).sort().join(', '));
  } else {
    console.log('None (already standard ESTree)');
  }

  // Validate before stripping
  const validationBefore = validateESTreeAST(ast);
  console.log('\n--- Validation Before Stripping ---');
  console.log('Valid ESTree:', validationBefore.valid);
  if (!validationBefore.valid) {
    console.log('Custom types:', validationBefore.customTypes.length);
  }

  // Strip custom nodes
  console.log('\n--- Stripping Custom Nodes ---');
  const { ast: strippedAst, stats } = stripCustomNodes(ast, {
    preserveLocations: true,
    verbose: false,
  });

  console.log('Nodes stripped:', stats.nodesStripped);
  console.log('Custom types removed:', Array.from(stats.customTypes).sort().join(', '));

  // Validate after stripping
  const validationAfter = validateESTreeAST(strippedAst);
  console.log('\n--- Validation After Stripping ---');
  console.log('Valid ESTree:', validationAfter.valid);
  if (!validationAfter.valid) {
    console.log('Remaining custom types:', validationAfter.customTypes);
    console.error('ERROR: Stripping failed to remove all custom nodes!');
  }

  // Save stripped AST
  const strippedPath = path.join(__dirname, `${filename}.stripped.ast.json`);
  fs.writeFileSync(
    strippedPath,
    JSON.stringify(strippedAst, null, 2),
    'utf8'
  );
  console.log('\nStripped AST saved to:', strippedPath);

  // Show stripped body structure
  console.log('\n--- Stripped AST Body Structure ---');
  if (strippedAst.body) {
    strippedAst.body.forEach((node, i) => {
      console.log(`[${i}] ${node.type}`);
      if (node.type === 'ExportDefaultDeclaration' && node.declaration) {
        console.log(`    ↳ exports: ${node.declaration.type}`);
        if (node.declaration.body?.body) {
          console.log(`    ↳ class body (${node.declaration.body.body.length} members):`);
          node.declaration.body.body.forEach((member, j) => {
            console.log(`       [${j}] ${member.type}${member.key?.name ? ` (${member.key.name})` : ''}`);
          });
        }
      }
    });
  }

  // Compare sizes
  const originalSize = JSON.stringify(ast).length;
  const strippedSize = JSON.stringify(strippedAst).length;
  const reduction = ((1 - strippedSize / originalSize) * 100).toFixed(1);

  console.log('\n--- Size Comparison ---');
  console.log('Original:', originalSize, 'bytes');
  console.log('Stripped:', strippedSize, 'bytes');
  console.log('Reduction:', reduction + '%');

  return {
    original: ast,
    stripped: strippedAst,
    stats,
    validationAfter,
  };
}

async function main() {
  const results = {
    gjs: await testStripper('sample.gjs'),
    gts: await testStripper('sample.gts'),
  };

  console.log('\n' + '='.repeat(80));
  console.log('Summary');
  console.log('='.repeat(80));

  console.log('\nGJS file:');
  console.log('  Custom nodes stripped:', results.gjs.stats.nodesStripped);
  console.log('  Valid after stripping:', results.gjs.validationAfter.valid);

  console.log('\nGTS file:');
  console.log('  Custom nodes stripped:', results.gts.stats.nodesStripped);
  console.log('  Valid after stripping:', results.gts.validationAfter.valid);

  const allValid = results.gjs.validationAfter.valid && results.gts.validationAfter.valid;

  if (allValid) {
    console.log('\n✅ SUCCESS: All custom nodes stripped, ASTs are valid ESTree!');
    console.log('\nNext step: Test conversion through oxc ESTree converter');
  } else {
    console.log('\n❌ FAILURE: Some custom nodes remain in the ASTs');
  }

  return results;
}

main().catch(console.error);
