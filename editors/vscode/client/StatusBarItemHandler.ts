import { MarkdownString, StatusBarAlignment, StatusBarItem, ThemeColor, window } from "vscode";

type StatusBarTool = "linter" | "formatter";

export default class StatusBarItemHandler {
  private tooltipSections: Map<StatusBarTool, string> = new Map();

  private statusBarItem: StatusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100);

  private extensionVersion: string = "<unknown>";

  constructor(extensionVersion?: string) {
    if (extensionVersion) {
      this.extensionVersion = extensionVersion;
    }
  }

  public show(): void {
    this.statusBarItem.show();
  }

  public setColorAndIcon(bgColor: string, icon: string): void {
    this.statusBarItem.backgroundColor = new ThemeColor(bgColor);
    this.statusBarItem.text = `$(${icon}) oxc`;
  }

  /**
   * Updates the tooltip text for a specific tool section.
   * The tooltip can use markdown syntax and VSCode icons.
   */
  public updateToolTooltip(toolId: StatusBarTool, text: string): void {
    this.tooltipSections.set(toolId, text);
    this.updateFullTooltip();
  }

  private updateFullTooltip(): void {
    const text = [this.tooltipSections.get("linter"), this.tooltipSections.get("formatter")]
      .filter(Boolean)
      .join("\n\n---\n\n");

    if (!(this.statusBarItem.tooltip instanceof MarkdownString)) {
      this.statusBarItem.tooltip = new MarkdownString("", true);
      this.statusBarItem.tooltip.isTrusted = true;
    }

    this.statusBarItem.tooltip.value = `VS Code Extension v${this.extensionVersion}\n\n---\n\n${text}`;
  }

  public dispose(): void {
    this.statusBarItem.dispose();
  }
}
