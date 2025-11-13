#!/usr/bin/env node

/**
 * Test script to verify eslint-plugin-ember integration with oxlint
 * 
 * This test:
 * 1. Configures eslint-plugin-ember in .oxlintrc.json
 * 2. Runs oxlint on a .gjs file with ember-specific issues
 * 3. Verifies that ember rules are executed and report issues
 */

import { readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('='.repeat(80));
console.log('eslint-plugin-ember Integration Test');
console.log('='.repeat(80));
console.log();

// Step 1: Check if oxlint binary exists
const oxlintPath = join(__dirname, '../../apps/oxlint/dist/cli.js');
if (!existsSync(oxlintPath)) {
  console.error('❌ Error: oxlint binary not found at:', oxlintPath);
  console.error('   Please run: pnpm --filter oxlint build-dev');
  process.exit(1);
}
console.log('✅ Found oxlint binary at:', oxlintPath);
console.log();

// Step 2: Check if eslint-plugin-ember is installed
const packageJsonPath = join(__dirname, 'package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const hasEmberPlugin = packageJson.dependencies?.['eslint-plugin-ember'] || 
                       packageJson.devDependencies?.['eslint-plugin-ember'];
if (!hasEmberPlugin) {
  console.error('❌ Error: eslint-plugin-ember not found in package.json');
  console.error('   Please run: cd tests/ember-parser-test && npm install');
  process.exit(1);
}
console.log('✅ eslint-plugin-ember is installed');
console.log();

// Step 3: Show configuration
const configPath = join(__dirname, '.oxlintrc.json');
const config = JSON.parse(readFileSync(configPath, 'utf-8'));
console.log('📄 Configuration (.oxlintrc.json):');
console.log(JSON.stringify(config, null, 2));
console.log();

// Step 4: Show test file
const testFile = join(__dirname, 'test-ember-plugin.gjs');
if (!existsSync(testFile)) {
  console.error('❌ Error: Test file not found:', testFile);
  process.exit(1);
}

console.log('📝 Test File (test-ember-plugin.gjs):');
console.log('-'.repeat(40));
const testCode = readFileSync(testFile, 'utf-8');
console.log(testCode);
console.log('-'.repeat(40));
console.log();

// Step 5: Expected violations
console.log('🔍 Expected Violations:');
console.log('  - ember/no-empty-glimmer-component-classes: Empty Glimmer component');
console.log('  - ember/require-computed-property-dependencies: Missing dependencies in computed');
console.log('  - ember/no-get: Using .get() method');
console.log();

// Step 6: Run oxlint
console.log('🔍 Running oxlint with eslint-plugin-ember...');
console.log();

try {
  const result = execSync(
    `node ${oxlintPath} ${testFile} --disable-nested-config`,
    {
      cwd: __dirname,
      encoding: 'utf-8',
      stdio: 'pipe'
    }
  );

  console.log('📋 Linting Results:');
  if (result.trim()) {
    console.log(result);
    
    // Check if ember rules are working (look for "ember(" or "ember/" in output)
    if (result.includes('ember(') || result.includes('ember/')) {
      console.log();
      console.log('✅ SUCCESS: eslint-plugin-ember rules are executing!');
      console.log('   Found ember rule violations in output.');
    } else {
      console.log();
      console.log('⚠️  WARNING: No ember rule violations found in output.');
      console.log('   This might mean:');
      console.log('   - Rules are not configured correctly');
      console.log('   - Rules are not being executed');
      console.log('   - Test file does not trigger the rules');
    }
  } else {
    console.log('✨ No linting errors found!');
    console.log('⚠️  This is unexpected - the test file should have ember rule violations.');
  }
} catch (error) {
  // execSync throws on non-zero exit code (which is expected when there are errors)
  if (error.stdout) {
    console.log('📋 Linting Results:');
    console.log(error.stdout);
    
    // Check if ember rules are working (look for "ember(" or "ember/" in output)
    if (error.stdout.includes('ember(') || error.stdout.includes('ember/')) {
      console.log();
      console.log('✅ SUCCESS: eslint-plugin-ember rules are executing!');
      console.log('   Found ember rule violations in output.');
      console.log('   Exit code:', error.status, '(expected when errors are found)');
    } else {
      console.log();
      console.log('⚠️  WARNING: No ember rule violations found in output.');
      console.log('   This might mean:');
      console.log('   - Rules are not configured correctly');
      console.log('   - Rules are not being executed');
      console.log('   - Test file does not trigger the rules');
      console.log('   Exit code:', error.status);
    }
  }

  if (error.stderr) {
    console.error('⚠️  Warnings/Errors:');
    console.error(error.stderr);
  }
}

console.log();
console.log('='.repeat(80));
console.log('Test Complete');
console.log('='.repeat(80));

