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

const formats = ["jpg", "webp", "heif", "avif", "jxl"];
const pixelArtFormats = ["png", "webp"];

var fileCache = {};

fs.readFile(pathFn.join(hexo.base_dir, "cache.image.json"), "utf8")
  .then((data) => {
    fileCache = JSON.parse(data);
  })
  .catch((_) => {});

function cache_key(args) {
  let inp = args.map((v) => JSON.stringify(v)).join("|");
  const hash = crypto.createHash("sha256").update(inp).digest("hex");
  return hash;
}

function lookup_cache(args) {
  return fileCache[cache_key(args)];
}

async function set_cache(args, result) {
  fileCache[cache_key(args)] = result;
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

async function do_one(srcPath, options, width, format, dest_dir, lossless) {
  let cached_result = lookup_cache([srcPath, options, width, format, lossless]);
  if (cached_result) return cached_result;
  console.log(`Converting ${srcPath} to ${width}x${width} with ${format}`);
  let out_path =
    cache_key([srcPath, options, width, format, lossless]) + `.${format}`;
  if (lossless) {
    await easyimage.execute("convert", [
      srcPath,
      ...(options.extraArgs || []),
      "-resize",
      `${width}x${width}`,
      out_path,
    ]);
  } else {
    await easyimage.execute("convert", [
      srcPath,
      ...(options.extraArgs || []),
      "-resize",
      `${width}x${width}`,
      "-quality",
      qualities[format],
      out_path,
    ]);
  }
  const hash = await hash_file(out_path);
  await fs.rename(out_path, pathFn.join(dest_dir, `${hash}.${format}`));
  console.log(`Converted ${srcPath} to ${hash}.${format}`);
  await set_cache([srcPath, options, width, format], hash);
  return hash;
}

async function convert_image(args, options) {
  console.log(args);
  var [src] = args;
  options = JSON.parse(options) || {};
  console.log(options);

  options.pixelArt = options.pixelArt || false;

  let chosenFormats = formats;
  if (options.pixelArt) {
    chosenFormats = pixelArtFormats;
  }

  let srcPath = pathFn.join(hexo.base_dir, "assets", src);
  if (!(await fs.pathExists(srcPath))) {
    srcPath = pathFn.join(hexo.source_dir, "_posts", src);
  }
  console.log(`Converting ${srcPath}`);
  const info = await easyimage.info(srcPath);
  console.log(info);

  const dest_dir = pathFn.join(hexo.public_dir, "img");
  await fs.mkdirs(dest_dir);

  let largestMediaWidth = Math.max(...options.widths.map(([a, _]) => a));

  var html = "<picture>";

  for (const format of chosenFormats) {
    for (const [mediaWidth, width] of options.widths) {
      if (width > info.width) continue;
      const hash = await do_one(
        srcPath,
        options,
        width,
        format,
        dest_dir,
        options.pixelArt
      );
      if (mediaWidth == largestMediaWidth) {
        html += `<source srcset="/img/${hash}.${format}" media="(min-width: ${mediaWidth}px)" type="image/${format}">`;
      } else {
        html += `<source srcset="/img/${hash}.${format}" media="(max-width: ${mediaWidth}px)" type="image/${format}">`;
      }
    }
  }

  if (options.pixelArt) {
    const hash = await do_one(
      srcPath,
      options,
      info.width,
      "png",
      dest_dir,
      true
    );
    html += `<img src="/img/${hash}.png" alt="${options.alt}" loading="lazy" decoding="lazy">`;
  } else {
    const hash = await do_one(
      srcPath,
      options,
      info.width,
      "jpg",
      dest_dir,
      false
    );
    html += `<img src="/img/${hash}.jpg" alt="${options.alt}" loading="lazy" decoding="lazy">`;
  }
  html += "</picture>";

  return html;
}

hexo.extend.tag.unregister("image");
hexo.extend.tag.register("image", convert_image, {
  async: true,
  ends: true,
});
