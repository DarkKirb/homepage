var fs = require("fs-extra");
var pathFn = require("path");
var easyimage = require("easyimage");
var crypto = require("crypto");

const qualities = {
  png: 100,
  jpg: 80,
  webp: 80,
  heif: 60,
  avif: 85,
  jxl: 70,
};

var fileCache = {};

fs.readFile(pathFn.join(hexo.base_dir, "cache.image.json"), "utf8")
  .then((data) => {
    fileCache = JSON.parse(data);
  })
  .catch((_) => {});

function lookup_cache(args) {
  return fileCache[args.map((v) => JSON.stringify(v)).join("|")];
}

async function set_cache(args, result) {
  fileCache[args.map((v) => JSON.stringify(v)).join("|")] = result;
  await fs.writeFile(
    pathFn.join(hexo.base_dir, "cache.image.json"),
    JSON.stringify(fileCache)
  );
}

async function hash_file(path) {
  let res = await fs.readFile(path, "binary");
  const hash = crypto.createHash("sha256").update(res).digest("hex");
  return hash;
}

async function do_one(srcPath, options, width, format, dest_dir) {
  let cached_result = lookup_cache([srcPath, options, width, format]);
  if (cached_result) return cached_result;
  console.log(`Converting ${srcPath} to ${width}x${width} with ${format}`);
  await easyimage.execute("convert", [
    srcPath,
    ...(options.extraArgs || []),
    "-resize",
    `${width}x${width}`,
    "-quality",
    qualities[format],
    `out.${format}`,
  ]);
  const hash = await hash_file(`out.${format}`);
  await fs.rename(`out.${format}`, pathFn.join(dest_dir, `${hash}.${format}`));
  console.log(`Converted ${srcPath} to ${hash}.${format}`);
  await set_cache([srcPath, options, width, format], hash);
  return hash;
}

async function convert_image(args, options) {
  console.log(args);
  var [src] = args;
  options = JSON.parse(options);
  console.log(options);

  const srcPath = pathFn.join(hexo.base_dir, "assets", src);
  console.log(`Converting ${srcPath}`);
  const info = await easyimage.info(srcPath);
  console.log(info);

  const dest_dir = pathFn.join(hexo.public_dir, "img");
  await fs.mkdirs(dest_dir);

  let largestMediaWidth = Math.max(...options.widths.map(([a, _]) => a));

  var html = "<picture>";

  for (const format of ["jxl", "avif", "heif", "webp", "jpg"]) {
    for (const [mediaWidth, width] of options.widths) {
      if (width > info.width) continue;
      const hash = await do_one(srcPath, options, width, format, dest_dir);
      if (mediaWidth == largestMediaWidth) {
        html += `<source srcset="/img/${hash}.${format}" media="(min-width: ${mediaWidth}px)" type="image/${format}">`;
      } else {
        html += `<source srcset="/img/${hash}.${format}" media="(max-width: ${mediaWidth}px)" type="image/${format}">`;
      }
    }
  }

  const hash = await do_one(srcPath, options, info.width, "jpg", dest_dir);
  html += `<img src="/img/${hash}.jpg" alt="${options.alt}">`;
  html += "</picture>";

  return html;
}

hexo.extend.tag.unregister("image");
hexo.extend.tag.register("image", convert_image, {
  async: true,
  ends: true,
});
