module.exports = {
  plugins: {
    "@csstools/postcss-sass": {
      includePaths: ["node_modules"],
    },
    cssnano: {
      preset: "advanced",
    },
    "postcss-prune-var": {},
  },
  syntax: "postcss-scss",
};
