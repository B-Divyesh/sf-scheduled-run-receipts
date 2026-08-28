import { readdir, readFile, stat, writeFile } from 'node:fs/promises';
import { join, relative, sep } from 'node:path';

const root = new URL('../dist/site/', import.meta.url);
const template = new URL('../site/public/sw.js', import.meta.url);

async function filesUnder(directory) {
  const files = [];
  for (const name of await readdir(directory)) {
    const path = join(directory, name);
    if ((await stat(path)).isDirectory()) files.push(...await filesUnder(path));
    else files.push(path);
  }
  return files;
}

const rootPath = root.pathname;
const files = (await filesUnder(rootPath))
  .filter((path) => !path.endsWith('/sw.js'))
  .map((path) => `/${relative(rootPath, path).split(sep).join('/')}`)
  .map((path) => path.endsWith('/index.html') ? path.slice(0, -10) : path);
const source = (await readFile(template, 'utf8'))
  .replace('"__SRR_PRECACHE__"', JSON.stringify(files));
await writeFile(new URL('sw.js', root), source);
