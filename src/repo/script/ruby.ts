import { Element } from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";

export const replaceRubyToRt = (
  text: string,
  rubies: [string, string][],
  paren: [string, string] = ["(", ")"]
): string => {
  return rubies.reduce(
    (text, [rb, rt]) => text.replace(`${rb}${paren[0]}${rt}${paren[1]}`, rt),
    text
  );
};

// <ruby><rb>漢字</rb><rp>(</rp><rt>かんじ</rt><rp>)</rp></ruby> -> かんじ
// <ruby><rb>成長</rb><rp>（</rp><rt>クルスト</rt><rp>）</rp></ruby>
export const extractRuby = (ruby: Element): [string, string] => {
  if (ruby.tagName !== "RUBY") {
    throw "must be ruby tag";
  }
  return [ruby.childNodes[0].textContent, ruby.childNodes[2].textContent];
};
