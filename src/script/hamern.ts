import {
  DOMParser,
  Element,
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";

export const hamern = (html: string): string[] => {
  const doc = new DOMParser().parseFromString(html, "text/html")!;
  const el = doc.querySelector("#honbun")!;
  return Array.from(el.children)
    .filter((e) => e.nodeName === "P")
    .map((e) =>
      replaceRubyToRt(
        e.textContent,
        Array.from(e.children)
          .filter((e) => e.tagName === "RUBY")
          .map(extractRuby)
      ).trim()
    )
    .filter((text) => text !== "");
};

const replaceRubyToRt = (text: string, rubies: [string, string][]): string => {
  return rubies.reduce(
    (text, [rb, rt]) => text.replace(`${rb}(${rt})`, rt),
    text
  );
};

// <ruby><rb>漢字</rb><rp>(</rp><rt>かんじ</rt><rp>)</rp></ruby> -> かんじ
const extractRuby = (ruby: Element): [string, string] => {
  if (ruby.tagName !== "RUBY") {
    throw "must be ruby tag";
  }
  return [ruby.childNodes[0].textContent, ruby.childNodes[2].textContent];
};
