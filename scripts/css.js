var postcss = require("postcss");
var postcssrc = require("postcss-load-config");

function renderer(data) {
  return postcssrc()
    .then(({ plugins, options }) => {
      options.from = data.path;
      return postcss(plugins).process(data.text, options);
    })
    .then((result) => result.css);
}

hexo.extend.renderer.register("css", "css", renderer);
hexo.extend.renderer.register("scss", "css", renderer);
