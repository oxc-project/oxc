import fs from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import process from 'node:process';
import vm from 'node:vm';

const __dirname = path.dirname(new URL(import.meta.url).pathname);
const harnessDir = path.join(__dirname, '../..', 'test262', 'harness');

const { Script, createContext, SourceTextModule, runInContext, SyntheticModule } = vm;

async function runCodeInHarness(options = {}) {
  const { code = '', includes = [], importDir = '', isAsync = false, isModule = true, isRaw = false } = options;
  const context = {};

  // See: https://github.com/nodejs/node/issues/36351
  const unique = () => '//' + Math.random();

  const runCode = async () => {
    const moduleCache = new Map();
    const dynamicImportCache = new Map();

    const findModule = (modulePath) => {
      let module = moduleCache.get(modulePath);
      if (!module) {
        const code = fs.readFileSync(modulePath, 'utf8');
        if (modulePath.endsWith('json')) {
          const evaluate = function() {
            this.setExport('default', runInContext('JSON.parse', context)(code));
          };
          module = new SyntheticModule(['default'], evaluate, { context });
        } else {
          module = new SourceTextModule(code + unique(), { context, importModuleDynamically });
        }
        moduleCache.set(modulePath, module);
      }
      return module;
    };

    const linker = (specifier, referencingModule) => {
      return findModule(path.join(importDir, specifier));
    };

    const importModuleDynamically = (specifier, script) => {
      const where = path.join(importDir, specifier);
      let promise = dynamicImportCache.get(where);
      if (!promise) {
        const module = findModule(where);
        if (module.status === 'unlinked') {
          promise = module.link(linker)
            .then(() => module.evaluate())
            .then(() => module);
        } else {
          promise = Promise.resolve(module);
        }
        dynamicImportCache.set(where, promise);
      }
      return promise;
    };

    createContext(context);
    if (!isRaw) runInContext(createHarnessForTest(includes), context);

    if (isModule) {
      const module = new SourceTextModule(code + unique(), { context, importModuleDynamically });
      await module.link(linker);
      await module.evaluate();
    } else {
      const script = new Script(code, { importModuleDynamically });
      script.runInContext(context);
    }
  };

  if (isAsync) {
    await new Promise((resolve, reject) => {
      context.$DONE = err => err ? reject(err) : resolve();
      runCode().catch(reject);
    });
  } else {
    await runCode();
  }
}

const harnessFiles = new Map();
let defaultHarness = '';

for (const entry of fs.readdirSync(harnessDir)) {
  if (entry.startsWith('.') || !entry.endsWith('.js')) {
    continue;
  }
  const file = path.join(harnessDir, entry);
  const content = fs.readFileSync(file, 'utf8');
  if (entry === 'assert.js' || entry === 'sta.js') {
    defaultHarness += content;
    continue;
  }
  harnessFiles.set(entry, content);
}

function createHarnessForTest(includes) {
  let harness = defaultHarness;

  if (includes) {
    for (const include of includes) {
      const content = harnessFiles.get(include);
      if (!content) throw new Error(`Included file is missing: ${include}`);
      harness += content;
    }
  }

  return harness;
}

const server = createServer((req, res) => {
  if (req.method === 'POST') {
    let body = '';
    req.on('data', chunk => {
      body += chunk.toString(); // convert Buffer to string
    });
    req.on('end', async () => {
      const options = JSON.parse(body);
      try {
        await runCodeInHarness(options);
      } catch (err) {
        if (parseInt(process.version.split('.')[0].replace('v', '')) < 20) {
          return res.end('Please upgrade the Node.js version to 20 or later.');
        }
        return res.end(err.toString());
      }

      // res.setHeader('Content-Type', 'application/json');
      res.end();
    });
  } else {
    res.statusCode = 404;
    res.end('Not Found');
  }
});

process.on('unhandledRejection', () => {
  // Don't exit when a test does this
});

server.listen(32055, () => {});
