const fs = require('node:fs');
const path = require('node:path');
const readline = require('node:readline');
const wkt = require('wkt');

(async () => {
  const input = process.argv[2];
  const outF = await fs.promises.open('output.csv', 'w');

  const rl = readline.createInterface({
    input: fs.createReadStream(input),
  });
  for await (const rawLine of rl) {
    const line = JSON.parse(rawLine);
    const properties = {...line.properties, ["建物ID"]: undefined};
    const propertiesStr = JSON.stringify(properties).replaceAll('"', '""');
    const wktPolygon = wkt.stringify(line.geometry);
    await outF.write(`"${line.properties["建物ID"]}","${wktPolygon}","${propertiesStr}"\n`);
  }
  await outF.close();
})();
