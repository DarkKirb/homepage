const pokemon = require("pokemon");

hexo.extend.tag.register("pokemon", (args) => {
  let id = pokemon.getId(args[0]);
  return `<img style="display: inline; margin-bottom: 0px;" src="https://mastodon-assets.chir.rs/mon/pokemon/icons/${id}.webp" height="22" role="presentation" loading="lazy" decoding="async" />${args[0]}`;
});

hexo.extend.tag.register("pokemonItem", (args) => {
  return `<img style="display: inline; margin-bottom: 0px;" src="https://mastodon-assets.chir.rs/mon/items/${args[0]}.webp" height="22" role="presentation" loading="lazy" decoding="async" />`;
});
