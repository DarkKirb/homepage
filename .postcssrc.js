module.exports = {
  plugins: {
    "@csstools/postcss-sass": {
      includePaths: ["node_modules"],
    },
    cssnano: {
      preset: "advanced",
    },
  },
  syntax: "postcss-scss",
};
