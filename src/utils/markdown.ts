import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import { type VNode, type VNodeArrayChildren, h } from "vue";

// Raw HTML stays disabled: docs come from shared files and must never inject markup.
const parser = new MarkdownIt({ html: false, linkify: true, breaks: false });

const BLOCK_TAGS = new Set([
  "p",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "ul",
  "ol",
  "li",
  "blockquote",
  "table",
  "thead",
  "tbody",
  "tr",
  "th",
  "td",
  "strong",
  "em",
  "s",
  "a",
]);

/** Links are only kept for web addresses; anything else renders as plain text. */
export const isWebLink = (href: string) => /^https?:\/\//i.test(href);

interface Frame {
  tag: string;
  props: Record<string, unknown>;
  children: VNodeArrayChildren;
}

function build(tokens: Token[]): VNodeArrayChildren {
  const root: Frame = { tag: "", props: {}, children: [] };
  const stack: Frame[] = [root];
  const top = () => stack[stack.length - 1];
  for (const token of tokens) {
    if (token.nesting === 1) {
      const props: Record<string, unknown> = {};
      const href = token.attrGet("href");
      if (token.tag === "a" && href && isWebLink(href)) {
        Object.assign(props, { href, "data-link": href, title: href });
      }
      const align = token.attrGet("style");
      if ((token.tag === "th" || token.tag === "td") && align?.startsWith("text-align:")) {
        props.style = align;
      }
      stack.push({ tag: BLOCK_TAGS.has(token.tag) ? token.tag : "span", props, children: [] });
      continue;
    }
    if (token.nesting === -1) {
      const frame = stack.pop()!;
      top().children.push(h(frame.tag, frame.props, frame.children));
      continue;
    }
    switch (token.type) {
      case "inline":
        top().children.push(...build(token.children ?? []));
        break;
      case "text":
        top().children.push(token.content);
        break;
      case "code_inline":
        top().children.push(h("code", token.content));
        break;
      case "code_block":
      case "fence":
        top().children.push(h("pre", h("code", token.content)));
        break;
      case "softbreak":
        top().children.push(" ");
        break;
      case "hardbreak":
        top().children.push(h("br"));
        break;
      case "hr":
        top().children.push(h("hr"));
        break;
      case "image":
        // Remote images are not loaded; the description stays readable.
        top().children.push(h("span", { class: "md-image" }, token.content || "gambar"));
        break;
    }
  }
  return root.children;
}

/** Markdown as Vue nodes, without any raw HTML from the source. */
export function renderMarkdown(source: string): VNode {
  return h("div", { class: "md" }, build(parser.parse(source, {})));
}
