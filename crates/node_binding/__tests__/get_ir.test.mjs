import test from 'ava';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturePath = join(__dirname, 'fixtures', 'petstore.json');

test('getIr returns valid GeneratorInput structure', async (t) => {
  const { getIr } = await import('../index.js');
  const ir = getIr(fixturePath);
  t.is(typeof ir, 'object');
  t.true(Array.isArray(ir.endpoints));
  t.is(typeof ir.project, 'object');
  t.is(ir.project.package_name, 'test-api');
});

test('getIr returns endpoints with correct fields', async (t) => {
  const { getIr } = await import('../index.js');
  const ir = getIr(fixturePath);
  t.true(ir.endpoints.length > 0);
  const ep = ir.endpoints[0];
  t.is(typeof ep.export_name, 'string');
  t.is(typeof ep.method, 'string');
  t.is(typeof ep.path, 'string');
});

test('getIr throws on invalid file path', async (t) => {
  const { getIr } = await import('../index.js');
  t.throws(() => getIr('/nonexistent/path.json'), {
    message: /Failed to read OpenAPI file/,
  });
});

test('getIr throws on invalid OpenAPI content', async (t) => {
  const { getIr } = await import('../index.js');
  const fs = await import('fs');
  const invalidPath = join(__dirname, 'fixtures', 'invalid.json');
  fs.writeFileSync(invalidPath, 'not valid json');
  try {
    t.throws(() => getIr(invalidPath));
  } finally {
    fs.unlinkSync(invalidPath);
  }
});
