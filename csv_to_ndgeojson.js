const fs = require('node:fs');
const path = require('node:path');
const readline = require('node:readline');
const csv = require('csv-parse');

(async () => {
  const input = process.argv[2];
  const outF = await fs.promises.open('output_from_csv.ndgeojson', 'w');

  const parser = csv.parse({
    columns: true,
  });
  const inputStream = fs.createReadStream(input);
  inputStream.pipe(parser);

  for await (const row of parser) {
    const [_pref_town_cd, _bldg, bldg_id] = row.id.split('-');
    await outF.write(JSON.stringify({
      type: 'Feature',
      geometry: JSON.parse(row.geometry),
      properties: JSON.parse(row.attributes),
      id: parseInt(bldg_id, 10),
    }) + '\n');
  }
  await outF.close();
})();
