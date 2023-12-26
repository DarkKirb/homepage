module.exports = {
  plugins: {
    "@csstools/postcss-sass": {
      includePaths: ["node_modules"],
    },
    "postcss-uncss": {
      html: ["public/**/*.html"],
      ignore: [
        "[data-theme=white]",
        "[data-theme=black]",
        "[data-theme=sunset]",
        "[data-theme=light]",
        '[data-theme="white"]',
        '[data-theme="black"]',
        '[data-theme="sunset"]',
        '[data-theme="light"]',
        ':root:is([data-theme="white"]):not([data-theme="black"]):not([data-theme="sunset"])',
      ],
      htmlroot: "public/",
      jsdom: {
        url: "https://lotte.chir.rs/",
      },
    },
    cssnano: {
      preset: "advanced",
    },
    "postcss-prune-var": {},
  },
  syntax: "postcss-scss",
};
