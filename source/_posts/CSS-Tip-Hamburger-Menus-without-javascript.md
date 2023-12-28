---
title: 'CSS Tip: Hamburger Menus without Javascript'
date: 2023-12-28 11:59:25
categories: webdev
tags: [webdev, css, frontend]
---

Expanding or slideover style menus are a popular way to create menus in newer websites, since they are friendly to touchscreen users (unlike menus that activate on hover).

I will show you how I have implemented the settings menu on this website.

<!-- more -->

For implementing a such a menu we use the fact that the `<input type="checkbox">` elements can be independently styled for their checked and unchecked states, as well as that you can use CSS to change the style on sibling elements using the [subsequent sibling (`~`) combinator](https://developer.mozilla.org/en-US/docs/Web/CSS/Subsequent-sibling_combinator).

<!-- toc -->

## My implementation

The HTML I use is:

```html
<input class="visually-hidden" type="checkbox" id="hamburger-menu-checkbox" aria-hidden="true" tabindex="-1">
<label class="hamburger" for="hamburger-menu-checkbox" title="Toggle Menu Name" aria-hidden="true">The Label for the Menu (an icon)</label>
<div id="hamburger-menu">
    Contents of the hamburger menu, hidden by default, but you could also reverse the logic by checking the input by default.
</div>
```

And the corresponding CSS:

```css
.visually-hidden,
#hamburger-menu-checkbox:not(:checked) ~ #hamburger-menu {
    clip: rect(0 0 0 0);
    clip-path: inset(50%);
    height: 1px;
    overflow: hidden;
    position: absolute;
    white-space: nowrap;
    width: 1px;
}
.hamburger {
  cursor: pointer;
  display: block;
  float: right;
  padding: 40px 20px;
}
```

## Explanation

On load the checkbox is not checked, and as such the selector `#hamburger-menu-checkbox:not(:checked)` will match the hamburger menu checkbox. The browser will then look for a sibling element with ID `hamburger-menu` following the checkbox, and hide it.

One particular feature of CSS checkboxes is that they change state when you press on the checkbox label, as such I have made the stylistic decision to hide the checkbox in the same manner using the visually-hidden class. I am not sure if the checkbox will work if it’s `display:none`, but I already have to have that particular CSS declaration as I will tell you later.

Once the checkbox becomes checked, the previous `:not(:checked)` will no longer match the checkbox, as such the hamburger menu will no longer be hidden.

## Further Work

Of course, just adding this snippet to your code will not instantly give the best experience. The menu pops in kind of suddenly and is entirely unanimated. As such you may want to play around with animating the max-height attribute on the `#hamburger-menu`.

## Accessiblity, or “Why am I not using `display:none`”

There is several ways to hide content from the user, namely:

- `display:none`: Your standard way of hiding content from the user. Unless the client has no CSS support, the element will be hidden completely.
- the `.visually-hidden` class from bootstrap: It will not display the element, but assistive technologies will still be able to read and interact with the element.
- the `aria-hidden` attribute: It will visually display the element, however assistive technologies will ignore it completely.

Due to how screen readers, for example, treat the website as a 1-Dimensional stream of words, it will be difficult for the user to notice what effect checking a checkbox will have on the rest of the site.

As such it is a good idea to make sure that the content of the hamburger menu is always available to the screen reader, but hidden from sighted users when collapsed. Similarly, I used `aria-hidden` on the control elements to reduce potential confusion about the purpose of those control elements.

## Browser Compatibility

As far as I can tell, any major browser, as well as Internet Explorer 9+, should support this.

## Browser Differences

Firefox will keep the value of form elements on reload, but not on shift-reload or page navigation. If you care enough about having the correct starting state, you can reset it in javascript.
