export interface MarkdownRenderOptions {
    linkAttributes?: Record<string, string>;
    externalLinkAttributes?: Record<string, string>;
    externalLinkIconHtml?: string;
}
export declare function markdownToHtml(markdown: string, options?: MarkdownRenderOptions): Promise<string | null>;
