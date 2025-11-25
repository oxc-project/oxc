import { ConfigurationChangeEvent, ExtensionContext, LogOutputChannel } from "vscode";
import { ConfigService } from "../ConfigService";
import StatusBarItemHandler from "../StatusBarItemHandler";

export default interface ToolInterface {
  /**
   * Gets the path to the tool's language server binary (if applicable).
   */
  getBinary(
    context: ExtensionContext,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
  ): Promise<string | undefined>;
  /**
   * Activates the tool and initializes any necessary resources.
   */
  activate(
    context: ExtensionContext,
    binaryPath: string,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void>;

  /**
   * Deactivates the tool and cleans up any resources.
   */
  deactivate(): Promise<void>;

  /**
   * Handles configuration changes.
   */
  onConfigChange(
    event: ConfigurationChangeEvent,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void>;
}
