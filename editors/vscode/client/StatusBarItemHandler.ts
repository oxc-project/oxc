import { MarkdownString, StatusBarAlignment, StatusBarItem, ThemeColor, window } from "vscode";

type StatusBarTool = "linter" | "formatter";

type ToolState = {
  isEnabled: boolean;
  content: string;
  version?: string;
};

export default class StatusBarItemHandler {
  private tooltipSections: Map<StatusBarTool, ToolState> = new Map([
    ["linter", { isEnabled: false, content: "", version: "unknown" }],
    ["formatter", { isEnabled: false, content: "", version: "unknown" }],
  ]);

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

  public setWarnBackground(): void {
    this.statusBarItem.backgroundColor = new ThemeColor("statusBarItem.warningBackground");
  }

  public setIcon(icon: string): void {
    this.statusBarItem.text = `$(${icon}) oxc`;
  }

  /**
   * Updates the tooltip text for a specific tool section.
   * The tooltip can use markdown syntax and VSCode icons.
   */
  public updateTool(
    toolId: StatusBarTool,
    isEnabled: boolean,
    text: string,
    version?: string,
  ): void {
    const section = this.tooltipSections.get(toolId);
    if (section) {
      section.isEnabled = isEnabled;
      section.content = text;
      section.version = version ?? "unknown";
      this.updateFullTooltip();
    }
  }

  private updateFullTooltip(): void {
    const sections: [string, ToolState][] = [
      ["oxlint", this.tooltipSections.get("linter")!],
      ["oxfmt", this.tooltipSections.get("formatter")!],
    ];

    const text = sections
      .map(([tool, section]) => {
        const version = section.version ? ` v${section.version}` : "unknown version";
        const statusText = section.isEnabled ? `enabled (${version})` : "disabled";
        return `**${tool} is ${statusText}**\n\n${section.content}`;
      })
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
