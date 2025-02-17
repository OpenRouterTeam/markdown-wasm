export interface MarkdownRenderOptions {
  linkAttributes?: Record<string, string>;
  externalLinkAttributes?: Record<string, string>;
  externalLinkIconHtml?: string;
}

export async function markdownToHtml(
  markdown: string,
  options: MarkdownRenderOptions = {}
): Promise<string | null> {
  const { markdown_to_html } = await import("../build");
  return markdown_to_html(markdown, options) ?? null;
}
