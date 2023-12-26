(() => {
  let theme = localStorage.getItem("theme");
  if (!theme) theme = "auto";
  let changeTheme = document.getElementById("change-theme");
  let changeThemeOptions = changeTheme.getElementsByTagName("option");
  for (let i = 0; i < changeThemeOptions.length; i++) {
    changeThemeOptions[i].selected = changeThemeOptions[i].value == theme;
  }
  changeTheme.addEventListener("change", (e) => {
    e.preventDefault();
    if (e.target.value == "auto") {
      localStorage.removeItem("theme");
      delete document.documentElement.dataset.theme;
    } else {
      localStorage.setItem("theme", e.target.value);
      document.documentElement.dataset.theme = e.target.value;
    }
  });
})();
