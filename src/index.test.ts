import { expect, test } from "vitest";

import { markdownToHtml } from "../dist";

test("adds attributes to links", async () => {
  const html = await markdownToHtml("[link](/here)", {
    linkAttributes: { class: "someclass" },
  });

  expect(html).toEqual('<p><a href="/here" class="someclass">link</a></p>\n');
});

test("adds extra attributes to external links", async () => {
  const html = await markdownToHtml("[link](https://google.com)", {
    linkAttributes: { class: "someclass" },
    externalLinkAttributes: { rel: "noopener noreferrer" },
  });

  expect(html).toEqual(
    '<p><a href="https://google.com" class="someclass" rel="noopener noreferrer">link</a></p>\n'
  );
});

test("doesn't add extra attributes to internal links", async () => {
  const html = await markdownToHtml("[link](/here)", {
    linkAttributes: { class: "someclass" },
    externalLinkAttributes: { rel: "noopener noreferrer" },
  });

  expect(html).toEqual('<p><a href="/here" class="someclass">link</a></p>\n');
});

test("adds icon to external links", async () => {
  const html = await markdownToHtml("[link](https://google.com)", {
    linkAttributes: { class: "someclass" },
    externalLinkAttributes: { rel: "noopener noreferrer" },
    externalLinkIconHtml: "<svg></svg>",
  });

  expect(html).toEqual(
    '<p><a href="https://google.com" class="someclass" rel="noopener noreferrer">link<svg></svg></a></p>\n'
  );
});

test("doesn't add icon to internal links", async () => {
  const html = await markdownToHtml("[link](/here)", {
    linkAttributes: { class: "someclass" },
    externalLinkAttributes: { rel: "noopener noreferrer" },
    externalLinkIconHtml: "<svg></svg>",
  });

  expect(html).toEqual('<p><a href="/here" class="someclass">link</a></p>\n');
});

test("aborts on raw html", async () => {
  const html = await markdownToHtml("<b>What?</b>");

  expect(html).toBe(null);
});

test("aborts on code blocks", async () => {
  const html = await markdownToHtml("```\nx = 3```");

  expect(html).toBe(null);
});

test("aborts on math blocks", async () => {
  const html = await markdownToHtml("$$\nx = 3\n$$");

  expect(html).toBe(null);
});

test("aborts on img tags", async () => {
  const html = await markdownToHtml("![image](https://google.com)");

  expect(html).toBe(null);
});
