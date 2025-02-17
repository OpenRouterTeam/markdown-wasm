export interface MarkdownRenderOptions {

}

export async function markdownToHtml(markdown: string, options: MarkdownRenderOptions = {}) {
    const { markdown_to_html } = await import('../build')
    return markdown_to_html(markdown, options);
}
