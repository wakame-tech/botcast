import R from "npm:remeda";
import {
  Element,
  NodeType,
} from "https://deno.land/x/deno_dom@v0.1.38/deno-dom-wasm.ts";

const ruby2rt = (e: Element): string => {
  if (e.tagName === "RUBY") {
    const rt = e.childNodes[2].textContent;
    return rt;
  } else {
    return e.textContent;
  }
};

export const filterEmptyLine = (lines: string[]) =>
  lines.filter((l) => l.trim() !== "");

export const extractPContents = (el: Element): string[] => {
  return R.pipe(
    Array.from(el.children),
    R.filter((e) => e.nodeName === "P"),
    R.map((p) => {
      return Array.from(p.childNodes)
        .map((n) => {
          if (
            n.nodeType === NodeType.ELEMENT_NODE &&
            (n as Element).tagName === "RUBY"
          ) {
            return ruby2rt(n as Element);
          } else {
            return n.textContent;
          }
        })
        .join("")
        .trim();
    }),
    filterEmptyLine
  );
};
