# OXC Minifier Examples

This directory contains practical examples demonstrating different aspects of the OXC JavaScript minifier.

## Available Examples

### `minifier.rs` - Complete Minification
Demonstrates the full minifier with both compression and mangling phases enabled.

**Features shown:**
- Complete minification pipeline
- Identifier mangling options
- Source map generation
- Stability testing with double minification

**Usage:**
```bash
cargo run --example minifier test.js
cargo run --example minifier --mangle --nospace --sourcemap test.js
```

### `dce.rs` - Dead Code Elimination  
Shows how to use only the compression phase for dead code elimination.

**Features shown:**
- Unreachable code removal
- Unused variable elimination  
- Conditional branch optimization
- Compression-only workflow

**Usage:**
```bash
cargo run --example dce test.js
cargo run --example dce --nospace --twice test.js
```

### `mangler.rs` - Identifier Shortening
Demonstrates using only the mangling phase to shorten identifiers.

**Features shown:**
- Variable and function name shortening
- Scope-aware renaming
- Debug mode for readable names
- Name preservation options

**Usage:**
```bash
cargo run --example mangler test.js
cargo run --example mangler --keep-names --debug test.js
```

## Creating Test Files

Create a `test.js` file in the project root with JavaScript code to minify:

```javascript
// test.js
function calculateTotal(items) {
    let total = 0;
    for (let i = 0; i < items.length; i++) {
        if (items[i].price > 0) {
            total += items[i].price;
        }
    }
    console.log('Total calculated:', total);
    return total;
}

// This code will never run
if (false) {
    console.log('Dead code');
}

const unusedVariable = 42;
```

## Understanding the Output

### Original Code (formatted)
```javascript
function calculateTotal(items) {
    let total = 0;
    for (let i = 0; i < items.length; i++) {
        if (items[i].price > 0) {
            total += items[i].price;
        }
    }
    console.log('Total calculated:', total);
    return total;
}
```

### After Dead Code Elimination
```javascript
function calculateTotal(items) {
    let total = 0;
    for (let i = 0; i < items.length; i++) {
        if (items[i].price > 0) {
            total += items[i].price;
        }
    }
    console.log('Total calculated:', total);
    return total;
}
// Dead code and unused variable removed
```

### After Mangling
```javascript
function calculateTotal(a) {
    let b = 0;
    for (let c = 0; c < a.length; c++) {
        if (a[c].price > 0) {
            b += a[c].price;
        }
    }
    console.log('Total calculated:', b);
    return b;
}
```

### After Complete Minification
```javascript
function calculateTotal(a){let b=0;for(let c=0;c<a.length;c++)a[c].price>0&&(b+=a[c].price);return console.log("Total calculated:",b),b}
```