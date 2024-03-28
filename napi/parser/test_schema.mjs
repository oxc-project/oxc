import {writeFile} from 'fs/promises';
import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';
import {getSchema} from './index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

const schemaArr = JSON.parse(getSchema());
const schema = Object.fromEntries(schemaArr.map(entry => [entry.name, entry]));
console.log(schema);

await writeFile(pathJoin(__dirname, 'schema.json'), JSON.stringify(schema, null, 2));

const sizes = [];
for (const entry of schemaArr) {
    sizes.push([entry.name, entry.size]);
}
sizes.sort(([name1], [name2]) => name1 < name2 ? -1 : 1);
await writeFile(pathJoin(__dirname, 'sizes.json'), JSON.stringify(Object.fromEntries(sizes), null, 2));
